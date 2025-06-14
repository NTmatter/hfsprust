# HFSPrust
Tools for reading (or attempting to read) damaged HFS+ volume images.

> **WARNING**: Only operates on raw disk images. Does not recover damaged disks, raid arrays, or other damaged sources.

# Reference
* [Technical Note TN1150 HFS Plus Volume Format](https://developer.apple.com/library/archive/technotes/tn/tn1150.html)
* [Mac OS 8 and 9 Developer Documentation](https://web.archive.org/web/19991001075851/http://developer.apple.com/techpubs/macos8/mac8.html) (wayback machine)
  * [Files](https://web.archive.org/web/19991001075851/http://developer.apple.com/techpubs/macos8/Files/files.html)
  * [File Manager](https://web.archive.org/web/19991001075851/http://developer.apple.com/techpubs/macos8/Files/FileManager/filemanager.html) ([pdf](https://vintageapple.org/inside_r/pdf/Files_1992.pdf))
* [apple-oss-distribution/hfs](https://github.com/apple-oss-distributions/hfs): Need to review the license, but it's probably worth porting the tests.
