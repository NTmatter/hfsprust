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
    let mut buf = [0u8; 1024];
    let Ok(()) = test_file.read_exact_at(&mut buf, 0) else {
        eprintln!("Failed to read leading zeroes before volume header");
        return;
    };

    if buf.into_iter().any(|x| x != 0) {
        eprintln!("Some bytes in pre-header were non-zero. Ignoring.");
    }

    // Read volume header structure.
    let mut buf = [0u8; 512];
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

    // Got volume header
    dbg!(&volume_header);

    // Extract useful information:
    println!("Sucessfully parsed volume header.");
    println!("Block Size: {}", &volume_header.block_size);
    // TODO Read Mounted/Unmounted attributes
}
