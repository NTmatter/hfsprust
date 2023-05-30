use deku::DekuContainerRead;
use hfsprust::*;
use std::env;
use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::os::unix::prelude::FileExt;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: read /path/to/file.img");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing file argument",
        ));
    }

    let test_path = args.get(1).unwrap();
    println!("Operating on {test_path}");

    let mut test_file = File::options().read(true).open(test_path)?;

    // Leading zeroes at start of file
    const PREAMBLE_LENGTH: usize = 1024;
    let mut buf = [0u8; PREAMBLE_LENGTH];
    test_file
        .read_exact_at(&mut buf, 0)
        .expect("1kB of zeroes present at start of volume");

    if buf.into_iter().any(|x| x != 0) {
        eprintln!("Some bytes in pre-header were non-zero. Ignoring.");
    }

    // Read volume header structure.
    const VOLUME_HEADER_LENGTH: usize = 512;
    let mut buf = [0u8; VOLUME_HEADER_LENGTH];
    test_file
        .read_exact_at(&mut buf, 1024)
        .expect("Read volume header");

    let (_rest, volume_header) = VolumeHeader::from_bytes((&buf, 0)).expect("Parse volume header");

    // Extract useful information:
    println!("Sucessfully parsed volume header.");
    let block_size = volume_header.block_size as usize;
    println!("Block Size: {}", block_size);
    // TODO Read Mounted/Unmounted attributes
    println!("Catalog File:");
    println!("blocks: {}", &volume_header.catalog_file.total_blocks);
    println!("logical_size: {}", &volume_header.catalog_file.logical_size);
    println!("clump_size: {}", &volume_header.catalog_file.logical_size);
    println!();
    println!("extent descriptor 0:");
    let catalog_start_block = volume_header.catalog_file.extents[0].start_block as usize;
    println!("start block: {}", catalog_start_block);
    let catalog_block_count = volume_header.catalog_file.extents[0].block_count as usize;
    println!("blocks: {}", catalog_block_count);

    let catalog_extents = assemble_extents(
        &mut test_file,
        &volume_header.catalog_file,
        volume_header.block_size as usize,
    )?;
    let mut cursor = Cursor::new(catalog_extents);

    read_btree(&mut cursor, volume_header.block_size as usize)?;

    let allocation_file = assemble_extents(
        &mut test_file,
        &volume_header.allocation_file,
        volume_header.block_size as usize,
    )?;

    println!("Allocation File Bitmap is {} bytes", allocation_file.len());

    Ok(())
}

fn read_btree_node(
    stream: &mut (impl Read + Seek),
    block_size: usize,
    record_size: usize,
) -> Result<(BTreeNodeDescriptor, Vec<u16>), io::Error> {
    // Consume entire record and operate on in-memory cursor.
    let mut record = vec![0u8; record_size];
    stream.read_exact(&mut record)?;

    let mut cursor = Cursor::new(record);

    // Read Node Descriptor
    let mut buf = [0; BTreeNodeDescriptor::SIZE];
    cursor.read_exact(&mut buf)?;
    let (_rest, node_descriptor) = BTreeNodeDescriptor::from_bytes((&mut buf, 0))?;

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

    // Extract records
    Ok((node_descriptor, offsets))
}

fn read_btree_header(
    stream: &mut (impl Read + Seek),
    block_size: usize,
) -> Result<(BTreeNodeDescriptor, BTreeHeaderRecord), io::Error> {
    // Read BTree Descriptor
    let mut buf = [0; BTreeNodeDescriptor::SIZE];
    stream.read_exact(&mut buf)?;
    let (_rest, node_descriptor) = BTreeNodeDescriptor::from_bytes((&mut buf, 0))?;

    // Read Header Record
    let mut buf = [0; BTreeHeaderRecord::SIZE];
    stream.read_exact(&mut buf)?;
    let (_rest, btree_header) = BTreeHeaderRecord::from_bytes((&mut buf, 0))?;

    // User Data is 128 bytes of reserved data. Skip it for now.
    let mut buf = [0; BTreeUserDataRecord::SIZE];
    stream.read_exact(&mut buf)?;
    let (_rest, _user_data) = BTreeUserDataRecord::from_bytes((&mut buf, 0))?;

    // The Map Record consumes all space until the record offsets at the end of the node.
    // This can be derived from the node size (specified in the node header) and the size
    // of all other structures (totals 256 bytes).
    let size_of_structures = 256;
    let map_record_size = btree_header.node_size - size_of_structures;
    let mut buf = vec![0u8; map_record_size as usize];
    stream.read_exact(&mut buf)?;

    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf)?;
    let _free_space_offset = u16::from_be_bytes(buf);

    // Parse offsets at end of header node
    let mut offsets = Vec::<u16>::with_capacity((node_descriptor.num_records) as usize);
    for _ in 0..(node_descriptor.num_records) {
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf)?;
        let offset = u16::from_be_bytes(buf);
        offsets.push(offset);
    }
    offsets.reverse();

    Ok((node_descriptor, btree_header))
}

fn read_btree(mut stream: &mut (impl Read + Seek), block_size: usize) -> Result<(), io::Error> {
    let (_node_descriptor, btree_header_record) = read_btree_header(&mut stream, block_size)?;
    let node_size = btree_header_record.node_size as usize;
    let total_nodes = btree_header_record.total_nodes as usize;

    // Read all nodes, skipping empties.
    for n in 1..total_nodes {
        let res = read_btree_node(&mut stream, block_size, node_size);
        if res.is_err() {
            eprintln!("Node {n} failed: {}", res.unwrap_err());
            continue;
        }

        let (node_header, offsets) = res.unwrap();
        if node_header.num_records > 0 {
            println!(
                "Node {n}   kind={:?}   records[{}]={:?}",
                node_header.kind,
                offsets.len(),
                offsets
            );
        }
    }

    todo!("Parse all BTree nodes")
}

/// Read all extents for a Fork. Does not handle Overflow extents.
fn assemble_extents(
    stream: &mut (impl Read + Seek),
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
        stream.seek(SeekFrom::Start(offset))?;
        stream.read_exact(buf)?;

        // Track bytes read.
        bytes_read += slice_length;
    }

    Ok(data)
}
