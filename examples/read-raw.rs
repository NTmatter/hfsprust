use deku::DekuContainerRead;
use hfsprust::raw::*;

use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read};

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: journal-info /path/to/file.img");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing file argument",
        ));
    }

    // Open file for reading
    let volume_file_path = args.get(1).expect("Path to image is first argument");
    println!("Processing file {volume_file_path}");

    let volume_file = File::options()
        .read(true)
        .open(volume_file_path)
        .expect("Open image for reading");

    let mut reader = BufReader::new(volume_file);

    // Volume preamble should be fully-zeroed.
    let mut buf = [0u8; 1024];
    reader
        .read_exact(&mut buf)
        .expect("Seek past empty volume preamble");

    if buf != [0u8; 1024] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Preamble was not fully-zeroed.",
        ));
    }

    // Read volume header
    let mut buf = [0u8; std::mem::size_of::<HFSPlusVolumeHeader>()];
    reader
        .read_exact(&mut buf)
        .expect("Read volume header bytes");

    let (_rest, volume_header) =
        HFSPlusVolumeHeader::from_bytes((&buf, 0)).expect("Parse volume header structure");

    // Transmute a copy, just to show off endianness issues.
    let transmuted: HFSPlusVolumeHeader = unsafe { std::mem::transmute(buf) };

    if volume_header.signature != transmuted.signature {
        eprintln!("Transmuted signature does not match endian-swapped signature.");
    }

    Ok(())
}
