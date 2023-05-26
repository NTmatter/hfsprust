#![forbid(unsafe_code)]
#![allow(dead_code)]

/// Unicode 2.0 String. Defined in TN1150 > HFS Plus Names.
/// Strings are stored fully-decomposed in canonical order.
struct HFSUniStr255 {
    length: u16,
    unicode: [u16; 255]
}

/// Encoding for conversion to MacOS-encoded Pascal String.
/// Defined in TN1150 > Text Encodings.
/// Representation is TBD
#[allow(clippy::enum_variant_names)]
enum TextEncodings {
    MacRoman = 0,
    MacJapanese = 1,
    MacChineseTriad = 2,
    MacKorean = 3,
    MacArabic = 4,
    MacHebrew = 5,
    MacGreek = 6,
    MacCyrillic = 7,
    MacDevanagari = 8,
    MacGurmukhi = 10,
    MacGujarati = 11,
    MacOriya = 12,
    MacBengali = 13,
    MacTamil = 14,
    MacTelugu = 15,
    MacKannada = 16,
    MacMalayalam = 17,
    MacSinhalese = 18,
    MacBurmese = 19,
    MacKhmer = 20,
    MacThai = 21,
    MacLaotian = 22,
    MacGeorgian = 23,
    MacArmenian = 24,
    MacChineseSimp = 25,
    MacTibetan = 26,
    MacMongolian = 27,
    MacEthiopic = 28,
    MacCentralEurRoman = 29,
    MacVietnamese = 30,
    MacExtArabic = 31,
    MacSymbol = 33,
    MacDingbats = 34,
    MacTurkish = 35,
    MacCroatian = 36,
    MacIcelandic = 37,
    MacRomanian = 38,
    MacFarsi = 49,
    MacFarsi2 = 140,
    MacUkrainian = 48,
    MacUkrainian2 = 152,
}

/// Dates are represented as seconds since Jan 1, 1904.
/// Defined in TN1150 > HFS Plus Dates
type Date = u32;

/// Type-dependent file information. Defined in `struct HFSPlusBSDInfo.special`
/// in TN1150 > HFS Plus Permissions.
union BsdInfoSpecial {
    inode_number: u32,
    link_count: u32,
    raw_device: u32,
}

/// File and Folder permissions. Defined as `struct HFSPlusBSDInfo` in
/// TN1150 > HFS Plus Permissions.
struct BsdInfo {
    owner_id: u32,
    group_id: u32,
    admin_flags: u8,
    owner_flags: u8,
    file_mode: u16,
    special: BsdInfoSpecial
}

// TODO Populate fileMode enum once it needs to be referenced
#[repr(u32)]
enum FileMode {
    Suid = 0o004000,
    Sgid = 0o002000,
    Sticky = 0o001000,
    
    OwnerRwxMask = 0o000700,
    OwnerR = 0o000400,
    OwnerW = 0o000200,
    OwnerX = 0o000100,
    
    GroupRwxMask = 0o000070,
    GroupR = 0o000040,
    GroupW = 0o000020,
    GroupX = 0o000010,
    
    OtherRwxMask = 0o000007,
    OtherR = 0o000004,
    OtherW = 0o000002,
    OtherX = 0o000001,

    FileTypeMask = 0o170000,
    Fifo = 0o010000,
    Character = 0o020000,
    Directory = 0o040000,
    Block = 0o060000,
    Regular = 0o100000,
    SymbolicLink = 0o120000,
    Socket = 0o140000,
    Whiteout = 0o160000,
}


/// Extent information. Defined as `struct HfsPlusExtentDescriptor` in
/// TN1150 > Fork Data Structure.
struct ExtentDescriptor {
    start_block: u32,
    block_count: u32,
}

/// When an extent descriptor is not used, it is set to zero.
const UNUSED_EXTENT_DESCRIPTOR: ExtentDescriptor = ExtentDescriptor { 
    start_block: 0,
    block_count: 0,
 };

/// A file's extent record is 8 
type ExtentRecord = [ExtentDescriptor; 8];

/// Resource and Data Fork contents. Defined as `struct HFSPlusForkData` in
/// TN1150 > Fork Data Structure.
struct ForkData {
    logical_size: u64,
    clump_size: u32,
    total_blocks: u32,
    extents: ExtentRecord,
}


/// Volume Signature, defined as `kHFSPlusSigWord` in TN1150 > Volume Header.
const VOLUME_SIGNATURE: [u8; 2] = [b'H', b'+'];

/// Known volume attribute bits. Defined as part of `struct HFSPlusVolumeHeader`
/// in TN1150 > Volume Header. Unknown bits MUST be zero.
#[repr(u32)]
enum VolumeAttributeBit {
    // Bits 0-6 are reserved
    // Documentation implies that 7 is reserved as well
    /// Volume is write-protected due to a hardware setting.
    /// NOTE: This is an assumption, as TN1150 does not document the bit.
    HardwareLock = 7,
    /// Volume successfully flushed during unmount. Set to 1 when unmounted.
    Unmounted = 8,
    /// Bad blocks are defined in Extents Overflow File.
    SparedBlocks = 9,
    /// Volume should not be cached in memory.
    NoCacheRequired = 10,
    /// Volume is currently mounted read-write. Set to zero when mounted RW.
    BootVolumeInconsistent = 11,
    /// NextCatalogNodeId has overflowed.
    CatalogNodeIdsReused = 12,
    /// Volume has a journal.
    Journaled = 13,
    // Bit 14 is reserved
    /// Volume is write-protected due to a software setting.
    SoftwareLock = 15,
    // Bits 16-31 are reserved
}


/// Volume Header, stored at 1024 bytes from start, and secondary header at 512
/// bytes from the end. Defined as `struct HFSPlusVolumeHeader` in
/// TN1150 > Volume Header.
struct VolumeHeader {
    signature: u16,
    version: u16,
    attributes: u32,
    last_mounted_version: u32,
    journal_info_block: u32,

    create_date: Date,
    modify_date: Date,
    backup_date: Date,
    checked_date: Date,

    file_count: u32,
    folder_count: u32,

    block_size: u32,
    total_blocks: u32,
    free_blocks: u32,

    next_allocation: u32,
    rsrc_clump_size: u32,
    data_clump_size: u32,
    next_catalog_id: CatalogNodeId,
    
    write_count: u32,
    encodings_bitmap: u64,

    finder_info: [u32; 8],

    allocation_file: ForkData,
    extents_file: ForkData,
    catalog_file: ForkData,
    attributes_file: ForkData,
    startup_file: ForkData,
}

/// Catalog Node ID or CNID identifies a B-tree file.
/// Defined in TN1150 > Catalog File.
type CatalogNodeId = u32;

#[allow(non_camel_case_types)]
#[repr(u32)]
enum StandardCnid {
    kHFSRootParentID            = 1,
    kHFSRootFolderID            = 2,
    kHFSExtentsFileID           = 3,
    kHFSCatalogFileID           = 4,
    kHFSBadBlockFileID          = 5,
    kHFSAllocationFileID        = 6,
    kHFSStartupFileID           = 7,
    kHFSAttributesFileID        = 8,
    kHFSRepairCatalogFileID     = 14,
    kHFSBogusExtentFileID       = 15,
    kHFSFirstUserCatalogNodeID  = 16,
}

struct CatalogFileKey {

}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
