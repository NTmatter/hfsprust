//! Unvalidated structures, suitable for mmapping or reading potentially
//! invalid data. Mostly copied and adapted from
//! [TN1150](https://developer.apple.com/library/archive/technotes/tn/tn1150.html),
//! retaining the original naming and layout.
//!

// Silence warnings caused by copying names from TN1150
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

#[repr(C, packed)]
pub struct HFSUniStr255<'a> {
    pub length: u16,
    pub unicode: &'a [u16], // This would be better off as a Vec<u16>.
}

/// An enum of encoding types to aid conversion.
pub type TextEncoding = u32;

/// Number of seconds since January 1, 1904, GMT.
pub type Date = u32;

#[repr(C, packed)]
pub struct HFSPlusBSDInfo {
    pub ownerID: u32,
    pub groupID: u32,
    pub adminFlags: u8,
    pub ownerFlags: u8,
    pub fileMode: u16,
    /// Originally a union for iNode, linkCount, rawDevice.
    pub special: u32,
}

// Wait for transparent unions: https://github.com/rust-lang/rust/issues/60405
// #[repr(C, transparent)]
pub union HFSPlusBSDInfo_special {
    pub iNodeNum: u32,
    pub linkCount: u32,
    pub rawDevice: u32,
}

#[repr(u32)]
pub enum FileMode {
    S_ISUID = 0o004000,
    S_ISGID = 0o002000,
    S_ISTXT = 0o001000,

    S_IRWXU = 0o000700,
    S_IRUSR = 0o000400,
    S_IWUSR = 0o000200,
    S_IXUSR = 0o000100,

    S_IRWXG = 0o000070,
    S_IRGRP = 0o000040,
    S_IWGRP = 0o000020,
    S_IXGRP = 0o000010,

    S_IRWXO = 0o000007,
    S_IROTH = 0o000004,
    S_IWOTH = 0o000002,
    S_IXOTH = 0o000001,

    S_IFMT = 0o170000,
    S_IFIFO = 0o010000,
    S_IFCHR = 0o020000,
    S_IFDIR = 0o040000,
    S_IFBLK = 0o060000,
    S_IFREG = 0o100000,
    S_IFLNK = 0o120000,
    S_IFSOCK = 0o140000,
    S_IFWHT = 0o160000,
}

#[repr(C, packed)]
pub struct HFSPlusForkData {
    pub logicalSize: u64,
    pub clumpSize: u32,
    pub totalBlocks: u32,
    pub extents: HFSPlusExtentRecord,
}

type HFSPlusExtentRecord = [HFSPlusExtentDescriptor; 8];

#[repr(C, packed)]
pub struct HFSPlusExtentDescriptor {
    pub startBlock: u32,
    pub blockCount: u32,
}

pub type HFSCatalogNodeID = u32;

#[repr(C, packed)]
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

    pub allocationFile: HFSPlusForkData,
    pub extentsFile: HFSPlusForkData,
    pub catalogFile: HFSPlusForkData,
    pub attributesFile: HFSPlusForkData,
    pub startupFile: HFSPlusForkData,
}

#[repr(u32)]
pub enum VolumeAttributeBit {
    // Bits 0-6 are reserved
    // Documentation implies that 7 is reserved as well
    /// Volume is write-protected due to a hardware setting.
    /// NOTE: This is an assumption, as TN1150 does not document the bit.
    kHFSVolumeHardwareLockBit = 7,
    /// Volume successfully flushed during unmount. Set to 1 when unmounted.
    kHFSVolumeUnmountedBit = 8,
    /// Bad blocks are defined in Extents Overflow File.
    kHFSVolumeSparedBlocksBit = 9,
    /// Volume should not be cached in memory.
    kHFSVolumeNoCacheRequiredBit = 10,
    /// Volume is currently mounted read-write. Set to zero when mounted RW.
    kHFSBootVolumeInconsistentBit = 11,
    /// NextCatalogNodeId has overflowed.
    kHFSCatalogNodeIDsReusedBit = 12,
    /// Volume has a journal.
    kHFSVolumeJournaledBit = 13,
    // Bit 14 is reserved.
    /// Volume is write-protected due to a software setting.
    kHFSVolumeSoftwareLockBit = 15,
    // Bits 16-31 are reserved
}

#[repr(C, packed)]
pub struct BTNodeDescriptor {
    fLink: u32,
    bLink: u32,
    kind: i8,
    height: u8,
    numRecords: u16,
    reserved: u16,
}

#[repr(i8)]
pub enum BTreeNodeKind {
    kBTLeafNode = -1,
    kBTIndexNode = 0,
    kBTHeaderNode = 1,
    kBTMapNode = 2,
}

#[repr(C, packed)]
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
    pub clumpSize: u32, // misaligned
    pub btreeType: u8,
    pub keyCompareType: u8,
    pub attributes: u32, // long aligned again
    pub reserved3: [u32; 16],
}

#[repr(u8)]
pub enum BTreeTypes {
    kHFSBTreeType = 0,    // control file
    kUserBTreeType = 128, // user btree type starts from 128
    kReservedBTreeType = 255,
}

#[repr(u32)]
pub enum BTHeaderRec_attributes {
    kBTBadCloseMask = 0x00000001,
    kBTBigKeysMask = 0x00000002,
    kBTVariableIndexKeysMask = 0x00000004,
}

pub const kHFSPlusCatalogMinNodeSize: u32 = 4096;
pub const kHFSPlusAttrMinNodeSize: u32 = 4096;

#[repr(u32)]
pub enum WellKnownCnid {
    kHFSRootParentID = 1,
    kHFSRootFolderID = 2,
    kHFSExtentsFileID = 3,
    kHFSCatalogFileID = 4,
    kHFSBadBlockFileID = 5,
    kHFSAllocationFileID = 6,
    kHFSStartupFileID = 7,
    kHFSAttributesFileID = 8,
    kHFSRepairCatalogFileID = 14,
    kHFSBogusExtentFileID = 15,
    kHFSFirstUserCatalogNodeID = 16,
}

#[repr(C, packed)]
pub struct HFSPlusCatalogKey<'a> {
    keyLength: u32,
    parentID: HFSCatalogNodeID,
    nodeName: HFSUniStr255<'a>,
}

pub const kHFSPlusCatalogKeyMinimumLength: u32 = 6;
pub const kHFSPlusCatalogKeyMaximumLength: u32 = 516;

#[repr(u16)]
pub enum BTNodeDescriptor_kind {
    kHFSPlusFolderRecord = 0x0001,
    kHFSPlusFileRecord = 0x0002,
    kHFSPlusFolderThreadRecord = 0x0003,
    kHFSPlusFileThreadRecord = 0x0004,
}

#[repr(C, packed)]
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

#[repr(C, packed)]
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

#[repr(u16)]
pub enum HFSPlusCatalog_fields {
    kHFSFileLockedBit = 0x0000,
    kHFSThreadExistsBit = 0x0001,
}

#[repr(u16)]
pub enum HFSPlusCatalog_masks {
    kHFSFileLockedMask = 0x0001,

    kHFSThreadExistsMask = 0x0002,
}

#[repr(C, packed)]
pub struct HFSPlusCatalogThread<'a> {
    pub recordType: i16,
    pub reserved: i16,
    pub parentID: HFSCatalogNodeID,
    pub nodeName: HFSUniStr255<'a>,
}

#[repr(C, packed)]
pub struct Point {
    pub v: i16,
    pub h: i16,
}

#[repr(C, packed)]
pub struct Rect {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16,
}

/* OSType is a 32-bit value made by packing four 1-byte characters
together. */
pub type FourCharCode = u32;
pub type OSType = FourCharCode;

/* Finder flags (finderFlags, fdFlags and frFlags) */
#[repr(u16)]
pub enum FinderFlags {
    kIsOnDesk = 0x0001, /* Files and folders (System 6) */
    kColor = 0x000E,    /* Files and folders */
    kIsShared = 0x0040, /* Files only (Applications only) If */
    /* clear, the application needs */
    /* to write to its resource fork, */
    /* and therefore cannot be shared */
    /* on a server */
    kHasNoINITs = 0x0080, /* Files only (Extensions/Control */
    /* Panels only) */
    /* This file contains no INIT resource */
    kHasBeenInited = 0x0100, /* Files only.  Clear if the file */
    /* contains desktop database resources */
    /* ('BNDL', 'FREF', 'open', 'kind'...) */
    /* that have not been added yet.  Set */
    /* only by the Finder. */
    /* Reserved for folders */
    kHasCustomIcon = 0x0400, /* Files and folders */
    kIsStationery = 0x0800,  /* Files only */
    kNameLocked = 0x1000,    /* Files and folders */
    kHasBundle = 0x2000,     /* Files only */
    kIsInvisible = 0x4000,   /* Files and folders */
    kIsAlias = 0x8000,       /* Files only */
}

/* Extended flags (extendedFinderFlags, fdXFlags and frXFlags) */
#[repr(u16)]
pub enum ExtendedFinderFlags {
    kExtendedFlagsAreInvalid = 0x8000, /* The other extended flags */
    /* should be ignored */
    kExtendedFlagHasCustomBadge = 0x0100, /* The file or folder has a */
    /* badge resource */
    kExtendedFlagHasRoutingInfo = 0x0004, /* The file contains routing */
                                          /* info resource */
}

#[repr(C, packed)]
pub struct FileInfo {
    pub fileType: OSType,    /* The type of the file */
    pub fileCreator: OSType, /* The file's creator */
    pub finderFlags: u16,
    pub location: Point, /* File's location in the folder. */
    pub reservedField: u16,
}

#[repr(C, packed)]
pub struct ExtendedFileInfo {
    pub reserved1: [i16; 4],
    pub extendedFinderFlags: u16,
    pub reserved2: i16,
    pub putAwayFolderID: i16,
}

#[repr(C, packed)]
pub struct FolderInfo {
    pub windowBounds: Rect, /* The position and dimension of the */
    /* folder's window */
    pub finderFlags: u16,
    pub location: Point, /* Folder's location in the parent */
    /* folder. If set to {0, 0}, the Finder */
    /* will place the item automatically */
    pub reservedField: u16,
}

#[repr(C, packed)]
pub struct ExtendedFolderInfo {
    pub scrollPosition: Point, /* Scroll position (for icon views) */
    pub reserved1: i32,
    pub extendedFinderFlags: u16,
    pub reserved2: i16,
    pub putAwayFolderID: i32,
}

#[repr(C, packed)]
pub struct HFSPlusExtentKey {
    pub keyLength: u16,
    pub forkType: u8,
    pub pad: u8,
    pub fileID: HFSCatalogNodeID,
    pub startBlock: u32,
}

fn IsAllocationBlockUsed(thisAllocationBlock: u32, allocationFileContents: &[u8]) -> bool {
    let thisByte: u8;

    thisByte = allocationFileContents[(thisAllocationBlock / 8) as usize];
    return (thisByte & (1 << (7 - (thisAllocationBlock % 8)))) != 0;
}

#[repr(u32)]
pub enum HFSPlusAttrForkData_recordType {
    kHFSPlusAttrInlineData = 0x10,
    kHFSPlusAttrForkData = 0x20,
    kHFSPlusAttrExtents = 0x30,
}

#[repr(C, packed)]
pub struct HFSPlusAttrForkData {
    pub recordType: u32,
    pub reserved: u32,
    pub theFork: HFSPlusForkData,
}

#[repr(C, packed)]
pub struct HFSPlusAttrExtents {
    pub recordType: u32,
    pub reserved: u32,
    pub extents: HFSPlusExtentRecord,
}

pub const kHardLinkFileType: &[u8] = b"hlnk"; // 0x686C6E6B
pub const kHFSPlusCreator: &[u8] = b"hfs+"; // 0x6866732B
pub const kSymLinkFileType: &[u8] = b"slnk"; // 0x736C6E6B
pub const kSymLinkCreator: &[u8] = b"rhap"; // 0x72686170

#[repr(C, packed)]
pub struct JournalInfoBlock {
    pub flags: u32,
    pub device_signature: [u32; 8],
    pub offset: u64,
    pub size: u64,
    pub reserved: [u32; 32],
}

#[repr(u32)]
pub enum JournalInfoBlock_flags {
    kJIJournalInFSMask = 0x00000001,
    kJIJournalOnOtherDeviceMask = 0x00000002,
    kJIJournalNeedInitMask = 0x00000004,
}

#[repr(C, packed)]
pub struct journal_header {
    pub magic: u32,
    pub endian: u32,
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub blhdr_size: u32,
    pub checksum: u32,
    pub jhdr_size: u32,
}

pub const JOURNAL_HEADER_MAGIC: u32 = 0x4a4e4c78;
pub const ENDIAN_MAGIC: u32 = 0x12345678;

#[repr(C, packed)]
pub struct block_list_header<'a> {
    pub max_blocks: u16,
    pub num_blocks: u16,
    pub bytes_used: u32,
    pub checksum: u32,
    pub pad: u32,
    pub binfo: &'a [block_info],
}

#[repr(C, packed)]
pub struct block_info {
    pub bnum: u64,
    pub bsize: u32,
    pub next: u32,
}

// Modified to operate on a slice, rather than pointer with offset.
fn calc_checksum(ptr: &[u8]) -> i32 {
    let mut cksum = 0i32;
    for b in ptr {
        cksum = (cksum << 8) ^ (cksum.overflowing_add(*b as i32).0);
    }

    return !cksum;
}

pub const HFC_MAGIC: u32 = 0xFF28FF26;
pub const HFC_VERSION: u32 = 1;
pub const HFC_DEFAULT_DURATION: u32 = 3600 * 60;
pub const HFC_MINIMUM_TEMPERATURE: u32 = 16;
pub const HFC_MAXIMUM_FILESIZE: u32 = 10 * 1024 * 1024;
pub const hfc_tag: &[u8] = b"CLUSTERED HOT FILES B-TREE     ";

#[repr(C, packed)]
struct HotFilesInfo<'a> {
    magic: u32,
    version: u32,
    duration: u32, /* duration of sample period */
    timebase: u32, /* recording period start time */
    timeleft: u32, /* recording period stop time */
    threshold: u32,
    maxfileblks: u32,
    maxfilecnt: u32,
    tag: &'a [u8; 32],
}

#[repr(C, packed)]
pub struct HotFileKey {
    pub keyLength: u16,
    pub forkType: u8,
    pub pad: u8,
    pub temperature: u32,
    pub fileID: u32,
}

const HFC_LOOKUPTAG: u32 = 0xFFFFFFFF;
const HFC_KEYLENGTH: usize = std::mem::size_of::<HotFileKey>() - std::mem::size_of::<u32>();

// TODO Case-insensitive string comparison. How does this compare to Rust's comparisons?
// I don't want to include the entire Unicode table, however it might be necessary to
// reuse it to ensure compatibility.

// TODO Add new parameters
// fn HFSPlusSectorToDiskSector(hfsPlusSector: u32) -> u32 {
//     let mut embeddedDiskOffset: u32;

//     embeddedDiskOffset = gMDB.drAlBlSt + gMDB.drEmbedExtent.startBlock * (drAlBlkSiz / 512);
//     return embeddedDiskOffset + hfsPlusSector;
// }
