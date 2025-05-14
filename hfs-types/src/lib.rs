// SPDX-License-Identifier: MIT

//! Types and constants from Apple's [TN1150 - HFS Plus Volume Format](https://developer.apple.com/library/archive/technotes/tn/tn1150.html)

#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]
#![forbid(dead_code, unsafe_code, unused)]

/// File and folder name, up to 255 Unicode-16 characters. Strings are stored as
/// fully decomposed in canonical order.
///
/// Described in TN1150 [HFS Plus Names](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HFSPlusNames)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSUniStr255 {
    pub length: u16,
    pub unicode: [u16; 255],
}

pub const kHFSPlusSigWord: u16 = u16::from_be_bytes(*b"H+");
pub const kHFSXSigWord: u16 = u16::from_be_bytes(*b"HX");

/// Catalog Node ID
///
/// Described in TN1150 [Catalog File](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFile)
pub type HFSCatalogNodeID = u32;

pub const kHFSRootParentID: u32 = 1;
pub const kHFSRootFolderID: u32 = 2;
pub const kHFSExtentsFileID: u32 = 3;
pub const kHFSCatalogFileID: u32 = 4;
pub const kHFSBadBlockFileID: u32 = 5;
pub const kHFSAllocationFileID: u32 = 6;
pub const kHFSStartupFileID: u32 = 7;
pub const kHFSAttributesFileID: u32 = 8;
pub const kHFSRepairCatalogFileID: u32 = 14;
pub const kHFSBogusExtentFileID: u32 = 15;
pub const kHFSFirstUserCatalogNodeID: u32 = 16;

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusVolumeHeader {
    pub signature: u16,
    pub version: u16,
    pub attributes: u32,
    pub lastMountedVersion: u32,
    pub journalInfoBlock: u32,

    pub createDate: u32,
    pub modifyDate: u32,
    pub backupDate: u32,
    pub checkedDate: u32,

    pub fileCount: u32,
    pub folderCount: u32,

    pub blockSize: u32,
    pub totalBlocks: u32,
    pub freeBlocks: u32,

    pub nextAllocation: u32,
    pub rsrcClumpSize: u32,
    pub dataClumpSize: u32,
    pub nextCatalogID: HFSCatalogNodeID,

    pub writeCount: u32,
    pub encodingsBitmap: u64,

    pub finderInfo: [u32; 8],
}

#[repr(u32)]
pub enum HFSPlusVolumeAttributeBit {
    // Bits 0-6 are reserved
    kHFSVolumeHardwareLockBit = 7,
    kHFSVolumeUnmountedBit = 8,
    kHFSVolumeSparedBlocksBit = 9,
    kHFSVolumeNoCacheRequiredBit = 10,
    kHFSBootVolumeInconsistentBit = 11,
    kHFSCatalogNodeIDsReusedBit = 12,
    kHFSVolumeJournaledBit = 13,
    // Bit 14 is reserved
    kHFSVolumeSoftwareLockBit = 15,
    // Bits 16-31 are reserved
}

/// Descriptor for fork data and location of core data.
///
/// Described by TN1150 in [Fork Data Structure](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ForkDataStructure)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusForkData {
    pub logicalSize: u64,
    pub clumpSize: u32,
    pub totalBlocks: u32,
    pub extents: HFSPlusExtentRecord,
}

/// Start and length of allocation block
///
/// Described by TN1150 in [Fork Data Structure](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ForkDataStructure)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusExtentDescriptor {
    /// First allocation block in the extent
    pub startBlock: u32,

    /// The length of the extent, in allocation blocks
    pub blockCount: u32,
}

pub type HFSPlusExtentRecord = [HFSPlusExtentDescriptor; 8];

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusBSDInfo {
    pub ownerID: u32,
    pub groupID: u32,
    pub adminFlags: u8,
    pub ownerFlags: u8,
    pub fileMode: u16,

    #[cfg(not(feature = "file_info_union"))]
    pub special: u32,

    #[cfg(feature = "file_info_union")]
    pub special: HFSPlusBSDInfoSpecial,
}

#[cfg(feature = "file_info_union")]
#[cfg_attr(feature = "repr_c", repr(C))]
pub union HFSPlusBSDInfoSpecial {
    pub iNodeNum: u32,
    pub linkCount: u32,
    pub rawDevice: u32,
}

pub const SF_ARCHIVED: u8 = 1;
pub const SF_IMMUTABLE: u8 = 2;
pub const SF_APPEND: u8 = 4;

pub const UF_NODUMP: u8 = 1;
pub const UF_IMMUTABLE: u8 = 2;
pub const UF_APPEND: u8 = 4;
pub const UF_OPAQUE: u8 = 8;

/// set user id on execution
pub const S_ISUID: u16 = 0o00_4000;

/// set group id on execution
pub const S_ISGID: u16 = 0o00_2000;

/// sticky bit
pub const S_ISTXT: u16 = 0o00_1000;

/// RWX mask for owner
pub const S_IRWXU: u16 = 0o00_0700;

/// R for owner
pub const S_IRUSR: u16 = 0o00_0400;

/// W for owner
pub const S_IWUSR: u16 = 0o00_0200;

/// X for owner
pub const S_IXUSR: u16 = 0o00_0100;

/// RWX mask for group
pub const S_IRWXG: u16 = 0o00_0070;

/// R for group
pub const S_IRGRP: u16 = 0o00_0040;

/// W for group
pub const S_IWGRP: u16 = 0o00_0020;

/// X for group
pub const S_IXGRP: u16 = 0o00_0010;

/// RWX mask for other
pub const S_IRWXO: u16 = 0o00_0007;

/// R for other
pub const S_IROTH: u16 = 0o00_0004;

/// W for other
pub const S_IWOTH: u16 = 0o00_0002;

/// X for other
pub const S_IXOTH: u16 = 0o00_0001;

/// type of file mask
pub const S_IFMT: u16 = 0o17_0000;

/// named pipe (fifo)
pub const S_IFIFO: u16 = 0o01_0000;

/// character special
pub const S_IFCHR: u16 = 0o02_0000;

/// directory
pub const S_IFDIR: u16 = 0o04_0000;

/// block special
pub const S_IFBLK: u16 = 0o06_0000;

/// regular
pub const S_IFREG: u16 = 0o10_0000;

/// symbolic link
pub const S_IFLNK: u16 = 0o12_0000;

/// socket
pub const S_IFSOCK: u16 = 0o14_0000;

/// whiteout
pub const S_IFWHT: u16 = 0o16_0000;

// region B-tree

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct BTNodeDescriptor {
    pub fLink: u32,
    pub bLink: u32,
    pub kind: BTNodeType,
    pub height: u8,
    pub numRecords: u16,
    pub reserved: u16,
}

#[repr(i8)]
pub enum BTNodeType {
    kBTLeafNode = -1,
    kBTIndexNode = 0,
    kBTHeaderNode = 1,
    kBTMapNode = 2,
}

#[cfg_attr(all(feature = "repr_c", not(feature = "packed_btree")), repr(C))]
#[cfg_attr(all(not(feature = "repr_c"), feature = "packed_btree"), repr(packed))]
#[cfg_attr(all(feature = "repr_c", feature = "packed_btree"), repr(C, packed))]
pub struct BTHeaderRec {
    pub treeDepth: u16,
    pub rootNode: u32,
    pub leafRecords: u32,
    pub firstLeafNode: u32,
    pub lastLeafNode: u32,
    pub nodeSize: u16,
    pub maxKeyLength: u16,
    pub totalNodes: u32,
    pub freeNodes: u32,
    pub reserved1: u16,
    // Misaligned
    pub clumpSize: u32,
    pub btreeType: u8,
    pub keyCompareType: u8,
    // long aligned again
    pub attributes: u32,
    pub reserved3: [u32; 16],
}

#[repr(u8)]
pub enum BTreeTypes {
    kHFSBTreeType = 0,    // control file
    kUserBTreeType = 128, // user btree type starts from 128
    kReservedBTreeType = 255,
}

#[repr(u32)]
pub enum BTreeHeaderRecAttribute {
    kBTBadCloseMask = 0x00000001,
    kBTBigKeysMask = 0x00000002,
    kBTVariableIndexKeysMask = 0x00000004,
}

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct BTreeUserDataRecord(pub [u8; 128]);

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct BTreeAllocationMapRecord(pub Vec<u8>);

pub fn IsAllocationBlockUsed(thisAllocationBlock: u32, allocationFileContents: &[u8]) -> bool {
    let thisByte = allocationFileContents[(thisAllocationBlock / 8) as usize];
    (thisByte & (1 << (7 - (thisAllocationBlock % 8)))) != 0
}

// endregion

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusCatalogKey {
    pub keyLength: u16,
    pub parentID: HFSCatalogNodeID,
    pub nodeName: HFSUniStr255,
}

/// BTree Record Type for a folder, to be interpreted as an `HFSPlusCatalogFolder`.
pub const kHFSPlusFolderRecord: u16 = 0x0001;

/// BTree Record Type for a file, to be interpreted as an `HFSPlusCatalogFile`.
pub const kHFSPlusFileRecord: u16 = 0x0002;

/// BTree Record Type for a folder thread record, to be interpreted as an `HFSPlusCatalogThread`.
pub const kHFSPlusFolderThreadRecord: u16 = 0x0003;

/// BTree Record Type for a file thread record, to be interpreted as an `HFSPlusCatalogThread`.
pub const kHFSPlusFileThreadRecord: u16 = 0x0004;

/// An on-screen point
///
/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct Point {
    pub v: i16,
    pub h: i16,
}

/// An on-screen rectangle
///
/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct Rect {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16,
}

pub type FourCharCode = u32;
pub type OSType = FourCharCode;

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusCatalogFolder {
    pub recordType: i16,
    pub flags: u16,
    pub valence: u32,
    pub folderID: HFSCatalogNodeID,
    pub createDate: u32,
    pub contentModDate: u32,
    pub attributeModDate: u32,
    pub accessDate: u32,
    pub backupDate: u32,
    pub permissions: HFSPlusBSDInfo,
    pub userInfo: FolderInfo,
    pub finderInfo: ExtendedFolderInfo,
    pub textEncoding: u32,
    pub reserved: u32,
}

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct FolderInfo {
    pub windowBounds: Rect,
    pub finderFlags: u16,
    pub location: Point,
    pub reservedField: u16,
}

pub const kHFSFolderRecord: u16 = 0x0100;
pub const kHFSFileRecord: u16 = 0x0200;
pub const kHFSFolderThreadRecord: u16 = 0x0300;
pub const kHFSFileThreadRecord: u16 = 0x0400;

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct ExtendedFolderInfo {
    pub scrollPosition: Point,
    pub reserved1: i32,
    pub extendedFinderFlags: u16,
    pub reserved2: i16,
    pub putAwayFolderID: u32,
}

/// B-tree record holding information about a file on the volume.
///
/// Described by TN1150 in [Catalog File Records](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFileRecord)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusCatalogFile {
    pub recordType: i16,
    pub flags: u16,
    pub reserved1: u32,
    pub fileID: HFSCatalogNodeID,
    pub createDate: u32,
    pub contentModDate: u32,
    pub attributeModDate: u32,
    pub accessDate: u32,
    pub backupDate: u32,
    pub permissions: HFSPlusBSDInfo,
    pub userInfo: FileInfo,
    pub finderInfo: ExtendedFileInfo,
    pub textEncoding: u32,
    pub reserved2: u32,

    pub dataFork: HFSPlusForkData,
    pub resourceFork: HFSPlusForkData,
}

pub const kHFSFileLockedBit: u16 = 0x0000;
pub const kHFSFileLockedMask: u16 = 0x0001;
pub const kHFSThreadExistsBit: u16 = 0x0001;
pub const kHFSThreadExistsMask: u16 = 0x0002;

/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct FileInfo {
    pub fileType: OSType,
    pub fileCreator: OSType,
    pub finderFlags: u16,
    pub location: Point,
    pub reserved: u16,
}

/// File type code for Hardlink files
///
/// Described by TN1150 in [Hard Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HardLinks)
pub const kHardLinkFileType: OSType = u32::from_be_bytes(*b"hlnk");

/// Creator code for Hardlink files
///
/// Described by TN1150 in [Hard Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HardLinks)
pub const kHFSPlusCreator: OSType = u32::from_be_bytes(*b"hfs+");

/// File type code for Symlink files
///
/// Described by TN1150 in [Symbolic Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#Symlinks)
pub const kSymLinkFileType: OSType = u32::from_be_bytes(*b"slnk");

/// Creator code for Symlink files
///
/// Described by TN1150 in [Symbolic Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#Symlinks)
pub const kSymLinkCreator: OSType = u32::from_be_bytes(*b"rhap");

/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct ExtendedFileInfo {
    pub reserved1: [i16; 4],
    pub extendedFinderFlags: u16,
    pub reserved2: i16,
    pub putAwayFolderID: i32,
}

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct HFSPlusCatalogThread {
    pub recordType: i16,
    pub reserved: i16,
    pub parentID: HFSCatalogNodeID,
    pub nodeName: HFSUniStr255,
}

/// Files and folders (System 6)
pub const kIsOnDesk: u16 = 0x0001;

/// Files and folders
pub const kColor: u16 = 0x000E;

/// Files only (Applications only) If clear, the application needs to write to its resource fork,
/// and therefore cannot be shared on a server
pub const kIsShared: u16 = 0x0040;

/// This file contains no INIT resource. Files only (Extensions/Control Panels only)
pub const kHasNoINITs: u16 = 0x0080;

/// Files only.  Clear if the file contains desktop database resources ('BNDL', 'FREF', 'open',
/// 'kind'...) that have not been added yet.  Set only by the Finder. Reserved for folders
pub const kHasBeenInited: u16 = 0x0100;

/// Files and folders
pub const kHasCustomIcon: u16 = 0x0400;

/// Files only
pub const kIsStationery: u16 = 0x0800;

/// Files and folders
pub const kNameLocked: u16 = 0x1000;

/// Files only
pub const kHasBundle: u16 = 0x2000;

/// Files and folders
pub const kIsInvisible: u16 = 0x4000;

/// Files only
pub const kIsAlias: u16 = 0x8000;

/// The other extended flags should be ignored
pub const kExtendedFlagsAreInvalid: u16 = 0x8000;

/// The file or folder has a badge resource
pub const kExtendedFlagHasCustomBadge: u16 = 0x0100;

/// The file contains routing info resource
pub const kExtendedFlagHasRoutingInfo: u16 = 0x0004;

pub struct HFSPlusExtentKey {
    pub key_length: u16,
    /// Type of fork for which this record applies. Must be 0x00 for the Data Fork
    /// and 0xFF for the resource fork.
    pub fork_type: u8,
    pub padding: u8,
    pub file_id: HFSCatalogNodeID,
    pub start_block: u32,
}

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct AttributeForkData {
    pub record_type: u32,
    pub reserved: u32,
    pub fork_data: HFSPlusForkData,
}

pub struct HFSPlusAttrExtents {
    pub recordType: u32,
    pub reserved: u32,
    pub extents: HFSPlusExtentRecord,
}

pub const kHFSPlusAttrInlineData: u32 = 0x10;
pub const kHFSPlusAttrForkData: u32 = 0x20;
pub const kHFSPlusAttrExtents: u32 = 0x30;
