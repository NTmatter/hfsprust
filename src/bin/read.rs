use deku::DekuContainerRead;
use hfsprust::*;
use std::env;
use std::fs::File;
use std::os::unix::prelude::FileExt;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: read /path/to/file.img");
        return;
    }

    let test_path = args.get(1).unwrap();
    println!("Operating on {test_path}");

    let Ok(test_file) = File::options().read(true).open(test_path) else {
        eprintln!("Failed to open file. Exiting.");
        return;
    };

    // Leading zeroes at start of file
    const PREAMBLE_LENGTH: usize = 1024;
    let mut buf = [0u8; PREAMBLE_LENGTH];
    let Ok(()) = test_file.read_exact_at(&mut buf, 0) else {
        eprintln!("Failed to read leading zeroes before volume header");
        return;
    };

    if buf.into_iter().any(|x| x != 0) {
        eprintln!("Some bytes in pre-header were non-zero. Ignoring.");
    }

    // Read volume header structure.
    const VOLUME_HEADER_LENGTH: usize = 512;
    let mut buf = [0u8; VOLUME_HEADER_LENGTH];
    let Ok(()) = test_file.read_exact_at(&mut buf, 1024) else {
        eprintln!("Failed to read volume header. Exiting.");
        return;
    };

    let parsed = VolumeHeader::from_bytes((&buf, 0));
    if parsed.is_err() {
        eprintln!("Failed to parse volume header: {}", parsed.unwrap_err());
        return;
    }
    let (_rest, volume_header) = parsed.unwrap();


    // Extract useful information:
    println!("Sucessfully parsed volume header.");
    let block_size = volume_header.block_size as usize;
    println!("Block Size: {}", block_size);
    // TODO Read Mounted/Unmounted attributes
    println!("Catalog File:");
    println!("         blocks: {}", &volume_header.catalog_file.total_blocks);
    println!("   logical_size: {}", &volume_header.catalog_file.logical_size);
    println!("     clump_size: {}", &volume_header.catalog_file.logical_size);
    println!("  extent descriptor 0:");
    let catalog_start_block = volume_header.catalog_file.extents[0].start_block as usize;
    println!("    start block: {}", catalog_start_block);
    let catalog_block_count = volume_header.catalog_file.extents[0].block_count as usize;
    println!("         blocks: {}", catalog_block_count);


    // Have a look at the Catalog File and try to examine the directory tree.
    // Assume that it's a single extent for now, noting that it is possible for
    // the file to be spread over multiple non-contiguous extents, and possibly
    // non-contiguous overflow extents as well.
    
    // Assume that all data is specified relative to the end of the first volume
    // header. 
    const VOLUME_DATA_START: usize = PREAMBLE_LENGTH + VOLUME_HEADER_LENGTH;

    let catalog_data_start: usize = VOLUME_DATA_START + catalog_start_block * block_size;
    let catalog_data_start: usize = PREAMBLE_LENGTH + catalog_start_block * block_size;
    let catalog_data_start: usize = catalog_start_block * block_size;

    // Try to have a look at the BTree Header Record
    const BTREE_HEADER_LENGTH: usize = 106;
    let mut buf = [0u8; BTREE_HEADER_LENGTH];
    let Ok(()) = test_file.read_exact_at(&mut buf, catalog_data_start as u64) else {
        eprintln!("Failed to read volume header. Exiting.");
        return;
    };

    let parsed = BTreeHeaderRecord::from_bytes((&mut buf, 0));
    if parsed.is_err() {
        eprintln!("Failed to parse BTree Header: {}", parsed.unwrap_err());
        return;
    }
    let (_rest, btree_header) = parsed.unwrap();
    
    dbg!(&btree_header);

}
