//! Locate the Journal file and print out some basic information.
//! Assumes that volume header and structures are sufficiently intact.
use deku::DekuContainerRead;
use hfsprust::*;
use std::{
    env,
    fs::File,
    io::{self, BufReader, Read, Seek},
};

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

    // Skip past volume preamble, assume zeroed.
    // TODO verify that volume header is all-zeroes.
    reader
        .seek(io::SeekFrom::Start(1024))
        .expect("Seek past empty volume preamble");

    // Read volume header
    let mut buf = [0u8; VolumeHeader::PACKED_SIZE];
    reader
        .read_exact(&mut buf)
        .expect("Read volume header bytes");

    let (_rest, volume_header) =
        VolumeHeader::from_bytes((&buf, 0)).expect("Parse volume header structure");

    // Calculate offset of Journal Info block
    let journal_info_block_offset =
        (volume_header.journal_info_block * volume_header.block_size) as u64;

    println!("Journal Info Block lives at {journal_info_block_offset:#X}");
    reader
        .seek(io::SeekFrom::Start(journal_info_block_offset))
        .expect("Seek to journal info block");

    let mut buf = [0u8; JournalInfoBlock::PACKED_SIZE];
    reader
        .read_exact(&mut buf)
        .expect("Read Journal Info Block bytes");

    let (_rest, journal_info_block) =
        JournalInfoBlock::from_bytes((&buf, 0)).expect("Parse Journal Header Block structure");

    println!("Journal is in-FS: {}", journal_info_block.flags.in_fs);
    println!(
        "Journal is located at {:#X}+{:#X}",
        journal_info_block.offset, journal_info_block.size
    );

    // WIP Did we land on the Journal magic?
    Ok(())
}
