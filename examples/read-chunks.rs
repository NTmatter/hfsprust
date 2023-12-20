//! How do we copy slices of a file without reading it into memory?
//! This might be doable with mmapped slices and IoSlices for write_vectored.
//! Chaining readers might still be the easiest option, though.
//!
//! Nope, there's no obvious way of specifying a length or end for a reader,
//! save for feeding it into a fixed-size buffer. This is problematic, as
//! extents can be extremely large.

use std::collections::BTreeMap;
use std::io::{Error, ErrorKind, IoSlice, Read, Write};

fn main() -> Result<(), std::io::Error> {
    // Can we create a reader starting at a particular byte with a syecific
    // length? It should be possible do do this
    Ok(())
}
