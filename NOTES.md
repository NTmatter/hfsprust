Consider using [memmap2](https://docs.rs/memmap2/latest/memmap2/struct.Mmap.html) to avoid reading file extents.

Consider using [Layered IO](https://docs.rs/layered-io/latest/layered_io/index.html) or [memoverlay](https://docs.rs/memoverlay/0.1.2/memoverlay/) atop memmap to nondestructively apply the journal.

Would the [Tokio Bytes](https://docs.rs/bytes/1.4.0/bytes/) crate be of use? Its underlying [rope](https://en.wikipedia.org/wiki/Rope_(data_structure)) implementation might be able to efficiently stitch extents together.


Would it be possible to assemble an overlay with Vectored IO? Is Vectored IO even seekable?
* [std::io::IoSlice](https://doc.rust-lang.org/std/io/struct.IoSlice.html)

File Assembly can definitely be rewritten with [Vectored IO](https://doc.rust-lang.org/std/io/trait.Write.html#method.write_vectored). That's a "Later" problem.

Consider applying Apache + MIT license.
* [HN Thread](https://news.ycombinator.com/item?id=21566968): Good GPLv2 compatibility. Recommends BSD+Patent.
* [Necessities - Rust API Guidelines](https://rust-lang.github.io/api-guidelines/necessities.html#crate-and-its-dependencies-have-a-permissive-license-c-permissive): Uses `license = "MIT OR Apache-2.0"`