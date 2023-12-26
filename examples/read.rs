use deku::bitvec::BitSlice;
use deku::DekuContainerRead;
use deku::DekuRead;
use hfsprust::*;
use itertools::Itertools;
use memmap2::UncheckedAdvice;
use memmap2::{Advice, MmapOptions};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::ErrorKind;
use std::io::IoSlice;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::os::unix::prelude::FileExt;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: read /path/to/file.img /path/to/output/");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing file argument",
        ));
    }

    let volume_file_path = args.get(1).expect("Path to Image File as first argument");
    println!("Operating on {volume_file_path}");
    let output_root_path = args
        .get(2)
        .expect("Path to output directory as second argument");
    println!("Writing to {output_root_path}");

    let mut volume_image = File::options()
        .read(true)
        .open(volume_file_path)
        .expect("Open volume image for reading");

    let volume_image_metadata = volume_image
        .metadata()
        .expect("Retrieve volume image file metadata");

    let volume_image_size = volume_image_metadata.len();

    // Leading zeroes at start of file
    const PREAMBLE_LENGTH: usize = 1024;
    let mut buf = [0u8; PREAMBLE_LENGTH];
    volume_image
        .read_exact_at(&mut buf, 0)
        .expect("1kB of zeroes present at start of volume");

    if buf.into_iter().any(|x| x != 0) {
        eprintln!("Some bytes in pre-header were non-zero. Ignoring.");
    }

    // Read volume header structure.
    const VOLUME_HEADER_LENGTH: usize = 512;
    let mut buf = [0u8; VOLUME_HEADER_LENGTH];
    volume_image
        .read_exact_at(&mut buf, 1024)
        .expect("Read volume header");

    let buf = BitSlice::from_slice(&buf);
    let (_rest, volume_header) = VolumeHeader::read(&buf, ()).expect("Parse volume header");

    // Extract useful information:
    println!("Sucessfully parsed volume header.");
    let block_size = volume_header.block_size as usize;
    println!("Block Size: {}", block_size);
    println!("Catalog File:");
    println!("\tblocks: {}", &volume_header.catalog_file.total_blocks);
    println!(
        "\tlogical_size: {}",
        &volume_header.catalog_file.logical_size
    );
    println!();

    let catalog_extents = assemble_extents(
        &mut volume_image,
        &volume_header.catalog_file,
        volume_header.block_size as usize,
    )?;
    println!("Assembled Catalog Extents.");
    println!("\tTotal Catalog Size: {}", &catalog_extents.len());

    let mut cursor = Cursor::new(catalog_extents);

    let map = read_btree_leaves(&mut cursor, volume_header.block_size as usize)?;

    // Generate list of all files and paths on volume, excluding HFS+ Private Data.
    println!("-- All Files --");
    let all_files = map
        .values()
        .filter_map(|record| match record {
            CatalogLeafRecord::File(file_record) => Some(cnid_to_key(file_record.file_id)),
            _ => None,
        })
        .map(|file_key| path_for_key(&map, file_key))
        .filter(|path| {
            !path.contains(&String::from("\0\0\0\0HFS+ Private Data"))
                && !path.contains(&String::from(".Spotlight-V100"))
        });
    all_files.for_each(|path| println!("{path:?}"));

    // Search for files that are spilling into Extents Overflow.
    println!("-- Overflow Files --");
    let overflow = map
        .values()
        .filter(|v| {
            if let CatalogLeafRecord::File(f) = v {
                f.data_fork.total_blocks
                    > f.data_fork
                        .extents
                        .iter()
                        .map(|extent| extent.block_count)
                        .sum()
            } else {
                false
            }
        })
        .collect_vec();
    println!("Overflow Files: {}", overflow.len());
    overflow.iter().for_each(|record| {
        if let CatalogLeafRecord::File(file_record) = record {
            println!("{:?}", path_for_key(&map, cnid_to_key(file_record.file_id)));
        }
    });

    // Extract all remaining non-overflow files.
    // Ensure output directory exists
    println!("-- Processing Files --");
    let output_root = PathBuf::from(output_root_path);
    if !output_root.exists() {
        fs::create_dir(&output_root)?;
    }

    if !output_root.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Output path is not a directory",
        ));
    }

    // This could probably be parallelized
    map.values()
        .filter(|record| matches!(record, CatalogLeafRecord::File(_)))
        .try_for_each(|record| {
            if let CatalogLeafRecord::File(file_record) = record {
                let file_key = cnid_to_key(file_record.file_id);
                let original_file_path = path_for_key(&map, file_key);

                // Skip HFS Private Data and various Metadata
                if original_file_path.contains(&String::from("\0\0\0\0HFS+ Private Data"))
                    // || original_file_path.contains(&String::from(".DS_Store"))
                    || original_file_path.contains(&String::from(".Spotlight-V100"))
                    || original_file_path.contains(&String::from(".journal_info_block"))
                    || original_file_path.contains(&String::from(".journal"))
                    || original_file_path.contains(&String::from(".fseventsd"))
                {
                    println!("Skipping {original_file_path:?}");
                    return Ok(());
                }

                // XXX Working with a truncated image, filter out any files with unavailable extents.
                for extent in &file_record.data_fork.extents {
                    let end_byte =
                        (extent.start_block + extent.block_count) as u64 * block_size as u64;
                    if end_byte > volume_image_size {
                        println!("Out of bounds {original_file_path:?}");
                        return Ok(());
                    }
                }

                println!(
                    "Processing {original_file_path:?} size={}",
                    file_record.data_fork.logical_size
                );

                // Create parent directories for output file if they do not exist
                let mut output_path = output_root.clone();
                original_file_path
                    .iter()
                    .for_each(|component| output_path.push(component));
                let parent_dir_path = output_path.parent().unwrap();
                if !parent_dir_path.exists() {
                    fs::create_dir_all(parent_dir_path)?;
                }

                let mut output_file = File::options()
                    .write(true)
                    .create_new(true)
                    .open(output_path)?;

                let res = copy_file_mmap(
                    &volume_image,
                    block_size as u16,
                    file_record,
                    Vec::new(),
                    &mut output_file,
                )?;
                println!(
                    "Done {original_file_path:?} bytes={} cksum={}",
                    res.0, res.1
                )
            }
            Ok::<(), io::Error>(())
        })?;

    Ok(())
}

fn cnid_to_key(cnid: CatalogNodeId) -> Vec<u8> {
    let mut key = Vec::<u8>::with_capacity(6);
    key.extend_from_slice(cnid.to_be_bytes().as_slice());
    key.extend(&[0u8; 2]);

    key
}

/// Construct a path for a given File Record
fn path_for_key(map: &BTreeMap<Vec<u8>, CatalogLeafRecord>, start: Vec<u8>) -> Vec<String> {
    // Record traversal to root
    let mut path = Vec::<String>::new();

    // Construct key for initial lookup
    let mut key = start;
    loop {
        if let Some(thread) = map.get(&key) {
            key = match thread {
                CatalogLeafRecord::Folder(_) => {
                    unreachable!("Unexpected folder record in thread!");
                }
                CatalogLeafRecord::File(_) => {
                    unreachable!("Unexpected file record in thread!");
                }
                CatalogLeafRecord::FolderThread(t) => {
                    let dir_name = String::from_utf16_lossy(&t.node_name.unicode);
                    path.push(dir_name);
                    cnid_to_key(t.parent_id)
                }
                CatalogLeafRecord::FileThread(t) => {
                    let file_name = String::from_utf16_lossy(&t.node_name.unicode);
                    path.push(file_name);
                    cnid_to_key(t.parent_id)
                }
            };
        } else {
            path.reverse();
            return path;
        };
    }
}

fn read_btree_node(
    stream: &mut (impl Read + Seek),
    _block_size: usize,
    record_size: usize,
) -> Result<(BTreeNodeDescriptor, Vec<Vec<u8>>), io::Error> {
    // Consume entire record and operate on in-memory cursor.
    let mut record = vec![0u8; record_size];
    stream.read_exact(&mut record)?;

    let mut cursor = Cursor::new(record);

    // Read Node Descriptor
    let mut buf = [0; BTreeNodeDescriptor::SIZE];
    cursor.read_exact(&mut buf)?;
    // let (_rest, node_descriptor) = BTreeNodeDescriptor::from_bytes((&mut buf, 0))?;
    let buf = BitSlice::from_slice(&buf);
    let (_rest, node_descriptor) = BTreeNodeDescriptor::read(&buf, ())?;

    // Read record offsets and free space offset from end of node.
    let offset_count = node_descriptor.num_records as usize + 1;
    let mut offsets = Vec::<u16>::with_capacity(offset_count);
    let seek_offset = record_size - BTreeNodeDescriptor::SIZE - 2 * offset_count;
    cursor.seek(SeekFrom::Current(seek_offset as i64))?;

    for _ in 0..=(node_descriptor.num_records) {
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf)?;
        let offset = u16::from_be_bytes(buf);
        offsets.push(offset);
    }
    offsets.reverse();

    // Extract record data
    let mut records = Vec::<Vec<u8>>::with_capacity(offsets.len() - 1);
    for (start, end) in offsets.into_iter().tuple_windows() {
        let len = end - start;
        let mut buf = vec![0u8; len as usize];
        cursor.seek(SeekFrom::Start(start as u64))?;
        cursor.read_exact(&mut buf)?;

        records.push(buf);
    }

    // Extract records
    Ok((node_descriptor, records))
}

/// Manually read the BTree header to bootstrap the rest of the read.
fn read_btree_header(
    stream: &mut (impl Read + Seek),
    _block_size: usize,
) -> Result<(BTreeNodeDescriptor, BTreeHeaderRecord), io::Error> {
    // Read BTree Descriptor
    let mut buf = [0; BTreeNodeDescriptor::SIZE];
    stream.read_exact(&mut buf)?;
    let buf = BitSlice::from_slice(&buf);

    let (_rest, node_descriptor) = BTreeNodeDescriptor::read(&buf, ())?;

    // Read Header Record
    let mut buf = [0; BTreeHeaderRecord::SIZE];
    stream.read_exact(&mut buf)?;
    let buf = BitSlice::from_slice(&buf);
    let (_rest, btree_header) = BTreeHeaderRecord::read(&buf, ())?;

    // User Data is 128 bytes of reserved data. Skip it for now.
    let mut buf = [0; BTreeUserDataRecord::SIZE];
    stream.read_exact(&mut buf)?;
    let (_rest, _user_data) = BTreeUserDataRecord::from_bytes((&mut buf, 0))?;

    // The Map Record consumes all space until the record offsets at the end of the node.
    // This can be derived from the node size (specified in the node header) and the size
    // of all other structures (totals 256 bytes).
    const MAP_STRUCTURE_SIZE: u16 = 256;
    let map_record_size = btree_header.node_size - MAP_STRUCTURE_SIZE;
    let mut buf = vec![0u8; map_record_size as usize];
    stream.read_exact(&mut buf)?;

    // Parse offsets at end of header node
    let mut offsets = Vec::<u16>::with_capacity((node_descriptor.num_records) as usize);
    for _ in 0..=node_descriptor.num_records {
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf)?;
        let offset = u16::from_be_bytes(buf);
        offsets.push(offset);
    }
    offsets.reverse();

    Ok((node_descriptor, btree_header))
}

fn read_btree_leaves(
    mut stream: &mut (impl Read + Seek),
    block_size: usize,
) -> Result<BTreeMap<Vec<u8>, CatalogLeafRecord>, io::Error> {
    let (_node_descriptor, btree_header_record) = read_btree_header(&mut stream, block_size)?;
    let node_size = btree_header_record.node_size as usize;
    let total_nodes = btree_header_record.total_nodes as usize;

    // TODO Consider restarting parse from header node

    // Read all nodes and extract leaves.
    let mut btree = BTreeMap::new();
    for n in 1..total_nodes {
        let res = read_btree_node(&mut stream, block_size, node_size);
        if res.is_err() {
            eprintln!("Node {n} failed: {}", res.unwrap_err());
            continue;
        }

        let (node_header, records) = res.unwrap();

        // Ignore empty nodes
        if node_header.num_records == 0 {
            continue;
        }

        // Print basic node information and record count
        // println!(
        //     "Node {n} - {:?}: {} Records",
        //     node_header.kind,
        //     records.len()
        // );

        // WIP: Focus on Leaf Nodes, as we're not trusting the on-disk b-tree for lookup.
        if node_header.kind != BTreeNodeKind::kBTLeafNode {
            continue;
        }

        records.iter().for_each(|record_bytes| {
            let (key, leaf_record) =
                parse_catalog_leaf(&record_bytes).expect("Parse catalog record");
            btree.insert(key, leaf_record);
        });
    }

    Ok(btree)
}

fn parse_catalog_leaf(record_bytes: &Vec<u8>) -> Result<(Vec<u8>, CatalogLeafRecord), io::Error> {
    let mut cur = Cursor::new(record_bytes);

    // Are we actually trying to read a Catalog File Key here?

    // Key Length: u16, as per TN1150 > Keyed Records. Might vary for non-leaves.
    let mut buf = [0u8; 2];
    cur.read_exact(&mut buf)
        .expect("Read length of catalog file key");
    let key_length = u16::from_be_bytes(buf);

    // Key Data (opaque)
    let mut key: BTreeKey = vec![0u8; key_length as usize];
    cur.read_exact(&mut key).expect("Read raw catalog file key");

    // Historical: Try to parse the key, as it actually has some structure.
    //
    // let mut key_cur = Cursor::new(&key);
    //
    // // Key: Parent CNID
    // let mut buf = [0u8; 4];
    // key_cur.read_exact(&mut buf)?;
    // let parent_cnid = u32::from_be_bytes(buf);
    //
    // // Key: String Length
    // let mut buf = [0u8; 2];
    // key_cur.read_exact(&mut buf)?;
    // let char_count = u16::from_be_bytes(buf) as usize;
    //
    // // Key: File Name
    // let mut name = Vec::<u16>::new();
    // for _ in 0..char_count {
    //     let mut buf = [0u8; 2];
    //     key_cur.read_exact(&mut buf)?;
    //     let char = u16::from_be_bytes(buf);
    //     name.push(char);
    // }
    // let name = String::from_utf16_lossy(&name);

    // Consume alignment/padding bytes if key length is odd.
    if key_length % 2 != 0 {
        // Consider: cur.consume(1);
        cur.read_exact(&mut [0u8; 1])
            .expect("Read padding byte from catalog leaf key");
    }

    let mut rest = Vec::<u8>::new();
    cur.read_to_end(&mut rest)?;

    // Peek at record kind
    let buf = vec![rest[0], rest[1]];
    let buf = BitSlice::from_slice(&buf);
    let (_rest, kind) = CatalogFileDataType::read(&buf, ())?;

    let rest = BitSlice::from_slice(&rest);

    // Parse payload
    // TODO Let Deku determine the variant type from the leading two bytes.
    let record = match kind {
        CatalogFileDataType::kHFSPlusFolderRecord => {
            let (_rest, folder) = CatalogFolder::read(&rest, ())?;
            CatalogLeafRecord::Folder(folder)
        }
        CatalogFileDataType::kHFSPlusFileRecord => {
            let (_rest, file) = CatalogFile::read(&rest, ())?;
            CatalogLeafRecord::File(file)
        }
        CatalogFileDataType::kHFSPlusFolderThreadRecord => {
            let (_rest, folder_thread) = CatalogThread::read(&rest, ())?;
            CatalogLeafRecord::FolderThread(folder_thread)
        }
        CatalogFileDataType::kHFSPlusFileThreadRecord => {
            let (_rest, file_thread) =
                CatalogThread::read(&rest, ()).expect("Parse File Thread Record");
            CatalogLeafRecord::FileThread(file_thread)
        }
    };

    Ok((key, record))
}

/// Concatenate all of a fork's extents into a single buffer. Does not handle Overflow extents yet.
fn assemble_extents(
    volume: &mut (impl Read + Seek),
    fork_data: &ForkData,
    block_size: usize,
) -> Result<Vec<u8>, io::Error> {
    let capacity = fork_data.logical_size as usize;

    if capacity == 0 {
        return Ok(Vec::<u8>::new());
    }

    let mut data = vec![0; capacity];

    let mut bytes_read = 0;
    for extent in &fork_data.extents {
        if extent.block_count == 0 {
            continue;
        }
        // Take fixed slice from data
        let slice_start = bytes_read;
        let slice_length = extent.block_count as usize * block_size;
        let slice_end = slice_start + slice_length;

        let buf = &mut data[slice_start..slice_end];

        let offset = extent.start_block as u64 * block_size as u64;
        volume.seek(SeekFrom::Start(offset))?;
        volume.read_exact(buf)?;

        // Track bytes read.
        bytes_read += slice_length;
    }

    Ok(data)
}

fn copy_file_mmap(
    volume: &File,
    block_size: u16,
    file_record: &CatalogFile,
    overflow_extents: Vec<ExtentDescriptor>,
    output: &mut impl Write,
) -> Result<(u64, String), io::Error> {
    let logical_size = file_record.data_fork.logical_size;
    let mmap = unsafe {
        MmapOptions::new()
            // .populate() // How much does this pre-populate?
            .map_copy_read_only(volume)
            .expect("foo")
    };

    let mut hasher = Sha256::new();

    if logical_size > 0 {
        // Build a list of ranges for vectored write
        let mut bytes_to_read = logical_size as usize;
        file_record
            .data_fork
            .extents
            .iter()
            .chain(overflow_extents.iter())
            .try_for_each(|extent| {
                let start_byte = extent.start_block as usize * block_size as usize;
                let length = extent.block_count as usize * block_size as usize;
                let length = if length > bytes_to_read {
                    bytes_to_read
                } else {
                    length
                };
                bytes_to_read = bytes_to_read
                    .checked_sub(length)
                    .expect("Do not read more than logical size");

                let end_byte = start_byte + length;

                // Suggest aggressive readahead and discard for anything larger than a few KB
                #[cfg(unix)]
                if length > 16_384 {
                    let _res = mmap.advise_range(Advice::Sequential, start_byte, length);
                }

                let slice = &mmap[start_byte..end_byte];
                hasher.update(&slice);
                let res = output.write_all(&slice);

                // Mark bytes as no-longer needed
                #[cfg(unix)]
                unsafe {
                    let _res =
                        mmap.unchecked_advise_range(UncheckedAdvice::DontNeed, start_byte, length);
                };

                res
            })
            .expect("Write all extents");

        assert_eq!(bytes_to_read, 0, "Must consume entire logical size of file");
    }

    let hash = format!("{:x}", hasher.finalize());
    Ok((logical_size, hash))
}

fn copy_file_data_from_extents(
    volume: &mut (impl Read + Seek),
    block_size: u16,
    file_record: &CatalogFile,
    overflow_extents: Vec<ExtentDescriptor>,
    output: &mut impl Write,
) -> Result<(u64, String), io::Error> {
    let logical_size = file_record.data_fork.logical_size;
    let mut bytes_read = 0u64;

    let mut hasher = Sha256::new();
    // Avoid work and corner cases for empty files.
    if logical_size == 0 {
        let hash = format!("{:x}", hasher.finalize());
        return Ok((logical_size, hash));
    }

    let mut volume = BufReader::with_capacity(1048576, volume);
    let mut output = BufWriter::with_capacity(1048576, output);

    file_record
        .data_fork
        .extents
        .iter()
        .chain(overflow_extents.iter())
        .try_for_each(|extent| {
            let source_start_byte = extent.start_block as u64 * block_size as u64;

            volume.seek(SeekFrom::Start(source_start_byte))?;
            for _ in extent.start_block..(extent.start_block + extent.block_count) {
                let mut buf = vec![0u8; block_size as usize];
                volume.read_exact(&mut buf)?;
                bytes_read += block_size as u64;

                // Trim any bytes that we don't need.
                if bytes_read > logical_size {
                    let residual = bytes_read - logical_size;
                    buf.truncate(residual as usize);
                }

                hasher.update(&buf);
                output.write_all(&buf)?;
            }

            Ok::<(), io::Error>(())
        })?;

    let hash = format!("{:x}", hasher.finalize());
    Ok((logical_size, hash))
}

fn copy_file_vectored(
    volume: &File,
    block_size: u16,
    file_record: &CatalogFile,
    overflow_extents: Vec<ExtentDescriptor>,
    output: &mut impl Write,
) -> Result<u64, io::Error> {
    let logical_size = file_record.data_fork.logical_size;
    let mmap = unsafe {
        MmapOptions::new()
            .populate()
            .map_copy_read_only(volume)
            .expect("foo")
    };

    // Build a list of ranges for vectored write
    let mut bytes_to_read = logical_size as usize;
    let extents: Vec<IoSlice> = file_record
        .data_fork
        .extents
        .iter()
        .chain(overflow_extents.iter())
        .map(|extent| {
            let start_byte = extent.start_block as usize * block_size as usize;
            let length = extent.block_count as usize * block_size as usize;
            let length = if length > bytes_to_read {
                bytes_to_read
            } else {
                length
            };
            bytes_to_read = bytes_to_read
                .checked_sub(length)
                .expect("Do not read more than logical size");

            let end_byte = start_byte + length;

            #[cfg(unix)]
            let _res = mmap.advise_range(Advice::Sequential, start_byte, length);
            let slice = &mmap[start_byte..end_byte];
            IoSlice::new(slice)
        })
        .collect();

    assert_eq!(bytes_to_read, 0, "Must consume entire logical size of file");

    // Does this actually handle partial writes correctly?
    let mut written = 0;
    while written < logical_size {
        match output.write_vectored(&extents) {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                written += n as u64;
            }
            Err(err) => {
                eprintln!("Failed to write: {err}");
                return Err(err);
            }
        }
    }

    Ok(logical_size)
}
