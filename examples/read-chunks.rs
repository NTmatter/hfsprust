//! How do we copy slices of a file without reading it into memory?
//! This might be doable with mmapped slices and IoSlices for write_vectored.
//! Chaining readers might still be the easiest option, though.
//!
//! Nope, there's no obvious way of specifying a length or end for a reader,
//! save for feeding it into a fixed-size buffer. This is problematic, as
//! extents can be extremely large.

use memmap2::{Advice, MmapOptions};
use std::env;
use std::fs::File;
use std::io::{self, Cursor, IoSlice, Write};

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

    // Surprisingly, a read-only map is safe! But is it usable?
    // I think I might have to do an unsafe map, like in the examples.
    let volume_mmap = unsafe {
        MmapOptions::new()
            .populate()
            .map_copy_read_only(&volume_file)
            .expect("Read-only mmap of volume")
    };

    volume_mmap
        .advise(Advice::Sequential)
        .expect("Failed to advise sequential access");

    // When does the actual read occur?
    let total_size = 12;
    let slices = vec![
        IoSlice::new(&volume_mmap[0..4]),
        IoSlice::new(&volume_mmap[4..8]),
        IoSlice::new(&volume_mmap[8..12]),
    ];

    let mut output = Vec::<u8>::new();
    let mut output = Cursor::new(&mut output);

    let mut written_bytes = 0;
    while written_bytes < total_size {
        let res = output.write_vectored(&slices);
        match res {
            Ok(0) => {
                println!("finished writing");
                break;
            }
            Ok(n) => {
                println!("Wrote {n} bytes");
                written_bytes += n;
            }
            Err(err) => eprintln!("Failed to write bytes: {err}"),
        }
    }

    Ok(())
}
