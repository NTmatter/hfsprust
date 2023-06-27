Consider a rename to `hfs_primitives` or `hfs_struct` and push HFS+ read/navigation functionality into a separate library.

Have a closer look at [ntfs](https://crates.io/crates/ntfs) crate for more inspiration.
* [ColinFinck/ntfs](https://github.com/ColinFinck/ntfs)
* Manages to do a `no_std` with alloc. How is this possible, doesn't `io::Error` preclude `no_std`?
  * Would it be possible to build a working reader with `no_alloc`, just using a fixed-size scratch? Zig would be a better candidate for this.

Consider using [memmap2](https://docs.rs/memmap2/latest/memmap2/struct.Mmap.html) to avoid reading file extents.

Consider using [Layered IO](https://docs.rs/layered-io/latest/layered_io/index.html) or [memoverlay](https://docs.rs/memoverlay/0.1.2/memoverlay/) atop memmap to nondestructively apply the journal.

Would the [Tokio Bytes](https://docs.rs/bytes/1.4.0/bytes/) crate be of use? Its underlying [rope](https://en.wikipedia.org/wiki/Rope_(data_structure)) implementation might be able to efficiently stitch extents together.


Would it be possible to assemble an overlay with Vectored IO? Is Vectored IO even seekable?
* [std::io::IoSlice](https://doc.rust-lang.org/std/io/struct.IoSlice.html)

File Assembly can definitely be rewritten with [Vectored IO](https://doc.rust-lang.org/std/io/trait.Write.html#method.write_vectored). That's a "Later" problem.

Consider applying Apache + MIT license.
* [HN Thread](https://news.ycombinator.com/item?id=21566968): Good GPLv2 compatibility. Recommends BSD+Patent.
* [Necessities - Rust API Guidelines](https://rust-lang.github.io/api-guidelines/necessities.html#crate-and-its-dependencies-have-a-permissive-license-c-permissive): Uses `license = "MIT OR Apache-2.0"`

BSD+Patent:
* [BSD+Patent](https://opensource.org/license/bsdpluspatent/)
* [Stack Overflow summary](https://opensource.stackexchange.com/a/9545): Needs a separate license for documentation. I'd prefer CC-BY-SA.
