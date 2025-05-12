#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]
#![deny(dead_code, unsafe_code)]

/// File and folder name, up to 255 Unicode-16 characters. Strings are stored as
/// fully decomposed in canonical order.
///
/// Described in TN1150 [HFS Plus Names](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HFSPlusNames)
#[repr(C)]
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

#[repr(C)]
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

#[repr(C)]
pub struct HFSPlusForkData {
    pub logicalSize: u64,
    pub clumpSize: u32,
    pub totalBlocks: u32,
    pub extents: [HFSPlusExtentDescriptor; 8]
}

#[repr(C)]
pub struct HFSPlusExtentDescriptor {
    pub startBlock: u32,
    pub blockCount: u32,
}

#[repr(C)]
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
#[repr(C)]
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
pub const S_ISUID: u16 = 0o0004000;
/// set group id on execution
pub const S_ISGID: u16 = 0o0002000;
/// sticky bit
pub const S_ISTXT: u16 = 0o0001000;
/// RWX mask for owner
pub const S_IRWXU: u16 = 0o0000700;
/// R for owner
pub const S_IRUSR: u16 = 0o0000400;
/// W for owner
pub const S_IWUSR: u16 = 0o0000200;
/// X for owner
pub const S_IXUSR: u16 = 0o0000100;
/// RWX mask for group
pub const S_IRWXG: u16 = 0o0000070;
/// R for group
pub const S_IRGRP: u16 = 0o0000040;
/// W for group
pub const S_IWGRP: u16 = 0o0000020;
/// X for group
pub const S_IXGRP: u16 = 0o0000010;
/// RWX mask for other
pub const S_IRWXO: u16 = 0o0000007;
/// R for other
pub const S_IROTH: u16 = 0o0000004;
/// W for other
pub const S_IWOTH: u16 = 0o0000002;
/// X for other
pub const S_IXOTH: u16 = 0o0000001;
/// type of file mask
pub const S_IFMT: u16 =   0o0170000;
/// named pipe (fifo)
pub const S_IFIFO: u16 =  0o0010000;
/// character special
pub const S_IFCHR: u16 =  0o0020000;
/// directory
pub const S_IFDIR: u16 =  0o0040000;
/// block special
pub const S_IFBLK: u16 =  0o0060000;
/// regular
pub const S_IFREG: u16 =  0o0100000;
/// symbolic link
pub const S_IFLNK: u16 =  0o0120000;
/// socket
pub const S_IFSOCK: u16 = 0o0140000;
/// whiteout
pub const S_IFWHT: u16 =  0o0160000;
