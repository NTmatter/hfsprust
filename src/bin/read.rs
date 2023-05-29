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

    dbg!(&volume_header);

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

    // Have a look at the Catalog File and try to examine the directory tree.
    // Assume that it's a single extent for now, noting that it is possible for
    // the file to be spread over multiple non-contiguous extents, and possibly
    // non-contiguous overflow extents as well.
    let catalog_data_start: usize = catalog_start_block * block_size;
    let btree_node_descriptor = read_node_descriptor(&test_file, catalog_data_start as u64);

    if btree_node_descriptor.is_err() {
        eprintln!(
            "Failed to read first btree node descriptor: {}",
            btree_node_descriptor.unwrap_err()
        );
    } else {
        dbg!(btree_node_descriptor.unwrap());
    }

    const BTREE_NODE_DESCRIPTOR_LENGTH: usize = 14;
    let btree_start = catalog_data_start + BTREE_NODE_DESCRIPTOR_LENGTH;

    let btree_header = read_btree_header(&test_file, btree_start as u64);
    if btree_header.is_err() {
        eprintln!("Failed to read btree header: {}", btree_header.unwrap_err());
    } else {
        dbg!(btree_header.unwrap());
    };

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

fn read_node_descriptor(file: &File, offset: u64) -> Result<BTreeNodeDescriptor, io::Error> {
    const BTREE_NODE_DESCRIPTOR_LENGTH: usize = 14;

    let mut buf = [0u8; BTREE_NODE_DESCRIPTOR_LENGTH];
    let _ = file.read_exact_at(&mut buf, offset)?;

    let (_rest, descriptor) = BTreeNodeDescriptor::from_bytes((&mut buf, 0))?;

    Ok(descriptor)
}

fn read_btree_header(file: &File, offset: u64) -> Result<BTreeHeaderRecord, io::Error> {
    // Try to have a look at the BTree Header Record
    const BTREE_HEADER_LENGTH: usize = 106;
    let mut buf = [0u8; BTREE_HEADER_LENGTH];
    let _ = file.read_exact_at(&mut buf, offset)?;

    let (_rest, btree_header) = BTreeHeaderRecord::from_bytes((&mut buf, 0))?;

    Ok(btree_header)
}

fn read_btree_node(stream: &mut (impl Read + Seek), block_size: usize) -> Result<(), io::Error> {
    // Read Nde Descriptor
    let mut buf = [0; BTreeNodeDescriptor::SIZE];
    stream.read_exact(&mut buf)?;
    let (_rest, node_descriptor) = BTreeNodeDescriptor::from_bytes((&mut buf, 0))?;

    Ok(())
}

fn read_btree(stream: &mut (impl Read + Seek), block_size: usize) -> Result<(), io::Error> {
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

    // Parse offsets at end of header node
    let mut offsets = Vec::<u16>::with_capacity((node_descriptor.num_records + 1) as usize);
    for _ in 0..(node_descriptor.num_records + 1) {
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf)?;
        let offset = u16::from_be_bytes(buf);
        offsets.push(offset);
    }
    offsets.reverse();

    Ok(())
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
