#![forbid(unsafe_code)]
#![allow(dead_code)]

/// Unicode 2.0 String. Defined in TN1150 > HFS Plus Names.
/// Strings are stored fully-decomposed in canonical order.
struct HFSUniStr255 {
    length: u16,
    unicode: [u16; 255],
}

/// Encoding for conversion to MacOS-encoded Pascal String.
/// Defined in TN1150 > Text Encodings.
#[repr(u32)]
#[allow(clippy::enum_variant_names)]
enum TextEncoding {
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
    special: BsdInfoSpecial,
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
#[derive(PartialEq)]
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

#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u32)]
enum StandardCnid {
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

/// BTree Node Descriptor.
/// Defined as `struct BTNodeDescriptor` in TN1150 > Node Structure.
struct BTreeNodeDescriptor {
    forward_link: u32,
    backward_link: u32,
    kind: BTreeNodeKind,
    height: u8,
    num_records: u16,
    reserved: u16,
}

/// Known values for BTreeNodeDescriptor::kind.
/// Defined in docs for `struct BTNodeDescriptor` in TN1150 > Catalog File.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(i8)]
enum BTreeNodeKind {
    kBTLeafNode = -1,
    kBTIndexNode = 0,
    kBTHeaderNode = 1,
    kBTMapNode = 2,
}

struct BTreeHeaderRecord {
    tree_depth: u16,
    root_node: u32,
    leaf_records: u32,
    first_leaf_node: u32,
    last_leaf_node: u32,
    node_size: u16,
    max_key_length: u16,
    total_nodes: u32,
    free_nodes: u32,
    reserved_1: u16,
    clump_size: u32,
    btree_type: BTreeType,
    key_compare_type: BTreeKeyCompareType,
    attributes: u32,
    reserved_3: [u32; 16],
}

/// Information about a catalog file.
/// Defined as `struct HFSPlusCatalogKey` in TN1150 > Catalog File.
struct CatalogFileKey {
    length: u16,
    parent: CatalogNodeId,
    name: HFSUniStr255,
}

/// Type of data contained in this catalog file.
/// Defined in documentation for `struct HFSPlusCatalogKey` in
/// TN1150 > Catalog File Data.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u16)]
enum CatalogFileDataType {
    kHFSPlusFolderRecord = 0x0001,
    kHFSPlusFileRecord = 0x0002,
    kHFSPlusFolderThreadRecord = 0x0003,
    kHFSPlusFileThreadRecord = 0x0004,
}

/// Type of data contained in this catalog file.
/// Defined in documentation for `struct HFSPlusCatalogKey` in
/// TN1150 > Catalog File Data.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u16)]
enum CatalogFolderDataType {
    kHFSFolderRecord = 0x0100,
    kHFSFileRecord = 0x0200,
    kHFSFolderThreadRecord = 0x0300,
    kHFSFileThreadRecord = 0x0400,
}

// Well-known values for BTreeNodeDescriptor.
/// Defined in documentation for `struct HFSPlusCatalogKey` in
/// TN1150 > Catalog File Data.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u8)]
enum BTreeType {
    kHFSBTreeType = 0,    // control file
    kUserBTreeType = 128, // user btree type starts from 128
    kReservedBTreeType = 255,
}

// Comparison mode, depending on HFSX support.
/// Defined in documentation for `struct HFSPlusCatalogKey` in
/// TN1150 > Catalog File Data.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u8)]
enum BTreeKeyCompareType {
    kHFSCaseFolding = 0xCF,
    kHFSBinaryCompare = 0xBC,
}

// TODO User Data Record: Second record in header node must be 128 bytes.

// TODO Map Record: Occupies remaining space in header node.

/// BTree leaf node for Folders. Defined as `struct HFSPlusCatalogFolder`
/// in TN1150 > Catalog Folder Records
struct CatalogFolder {
    /// Always CatalogFolderDataType::kHFSPlusFolderRecord
    record_type: CatalogFolderDataType,
    flags: u16,
    valence: u32,
    folder_id: CatalogNodeId,
    create_date: Date,
    content_mod_date: Date,
    attribute_mod_date: Date,
    access_date: Date,
    backup_date: Date,
    permissions: BsdInfo,
    user_info: FolderInfo,
    ifnder_info: ExtendedFolderInfo,
    text_encoding: TextEncoding,
    reserved: u32,
}


/// Defined in documentation for `struct HFSPlusCatalogFile` in
/// TN1150 > Catalog File Records. 
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u16)]
enum CatalogFileBit {
    kHFSFileLockedBit       = 0x0000,
    kHFSThreadExistsBit     = 0x0001,
}

/// Defined in documentation for `struct HFSPlusCatalogFile` in
/// TN1150 > Catalog File Records.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u16)]
enum CatalogFileBitMask {
    kHFSFileLockedMask       = 0x0001,
    kHFSThreadExistsMask     = 0x0002,
}

/// BTree leaf node for Files. Defined as `struct HFSPlusCatalogFile` in
/// TN1150 > Catalog File Records
struct CatalogFile {
    record_type: CatalogFileDataType,
    /// 
    flags: u16,
    reserved_1: u32,
    file_id: CatalogNodeId,
    create_date: Date,
    content_mod_date: Date,
    attribute_mod_date: Date,
    backup_date: Date,
    permissions: BsdInfo,
    finder_info: ExtendedFileInfo,
    text_encoding: TextEncoding,
    reserved_2: u32,

    data_fork: ForkData,
    resource_fork: ForkData,
}

/// A location on screen, used to store window placement.
/// Defined in TN1150 > Finder Info.
struct Point {
    v: i16,
    h: i16,
}

/// Rectangular region used for Directory windows.
/// Defined in TN1150 > Finder Info.
struct Rect {
    top: i16,
    left: i16,
    bottom: i16,
    right: i16,
}

/// Four characters representing OS used to write data.
/// Defined in TN1150 > Finder Info.
type OSType = u32;

/// Presentation info for Finder.
/// Defined in TN1150 > Finder Info.
struct FileInfo {
    file_type: OSType,
    file_creator: OSType,
    finder_flags: u16,
    location: Point,
    reserved: u16,
}

/// Additional file information for display in Finder
/// Defined in TN1150 > Finder Info.
struct ExtendedFileInfo {
    reserved_1: [i16; 4],
    extended_finder_flags: u16,
    reserved_2: i16,
    put_away_folder_id: i32,
}

/// Known flags for Finder
/// Defined in TN1150 > Finder Info.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u16)]
enum FileInfoFinderFlags {
    /// Files and Folders (System 6)
    kIsOnDesk = 0x0001,
    /// Files and Folders
    kColor = 0x000E,
    /// Files only (Applications only). If clear, the application needs
    /// to write to its resource fork, and therefore cannot be shared
    /// on a server.
    kIsShared = 0x0040,
    /// Files only (Extensions/Control Panels only). This file contains
    /// no INIT resource.
    kHasNoINITs = 0x0080,
    /// Files only. Clear if the file contains desktop database resources
    /// ('BNDL', 'FREF', 'open', 'kind' ...) that have not been added yet.
    /// Set only by he finder. Reserved for folders.
    kHasBeenInited = 0x0100,
    /// Files and folders
    kHasCustomIcon = 0x0400,
    /// Files only.
    kIsStationery = 0x0800,
    /// Files and folders
    kNameLocked = 0x1000,
    /// Files only.
    kHasBundle = 0x2000,
    /// Files and folders
    kIsInvisible = 0x4000,
    /// Files only.
    kIsAlias = 0x8000,
}

/// Finder Metadata and display information
/// Defined in TN1150 > Finder Info.
struct FolderInfo {
    window_bounds: Rect,
    finder_flags: u16,
    location: Point,
    reserved: u16,
}

/// Finder Metadata and display information
/// Defined in TN1150 > Finder Info.
struct ExtendedFolderInfo {
    scroll_position: Point,
    reserved_1: i32,
    extened_finder_flags: u16,
    reserved_2: i16,
    put_away_folder_id: i32,
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
