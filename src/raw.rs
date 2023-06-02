//! Unvalidated structures, suitable for mmapping or reading potentially
//! invalid data. Mostly copied and adapted from
//! [TN1150](https://developer.apple.com/library/archive/technotes/tn/tn1150.html),
//! retaining the original naming and layout.

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
