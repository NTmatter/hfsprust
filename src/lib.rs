#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod raw;

#[cfg(feature = "deku")]
use deku::ctx::Endian;
#[cfg(feature = "deku")]
use deku::prelude::*;
use std::io;
use std::io::{Cursor, Read};

/// Unicode 2.0 String. Defined in TN1150 > HFS Plus Names.
/// Strings are stored fully-decomposed in canonical order.
#[cfg(feature = "deku")]
#[cfg_attr(feature = "deku", deku_derive(DekuRead))]
#[deku(endian = "endian", ctx = "endian: Endian")]
pub struct HFSUniStr255 {
    #[deku(temp)]
    pub length: u16,
    #[deku(count = "length")]
    pub unicode: Vec<u16>,
}

// Manual reimplementation to handle issues with `#[deku(temp)]` macro.
// See https://github.com/sharksforarms/deku/issues/343
#[cfg(not(feature = "deku"))]
pub struct HFSUniStr255 {
    pub length: u16,
    pub unicode: Vec<u16>,
}

impl Into<String> for HFSUniStr255 {
    fn into(self) -> String {
        String::from_utf16_lossy(self.unicode.as_slice())
    }
}

/// Encoding for conversion to MacOS-encoded Pascal String.
/// Defined in TN1150 > Text Encodings.
#[repr(u32)]
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "big", type = "u32"))]
pub enum TextEncoding {
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
///
/// Unlike the spec, do not use a union to represent the different types of
/// special info.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct BsdInfoSpecial {
    /// May represent an inode number, link count, or raw device
    special: u32,
    // inode_number: u32,
    // link_count: u32,
    // raw_device: u32,
}

/// File and Folder permissions. Defined as `struct HFSPlusBSDInfo` in
/// TN1150 > HFS Plus Permissions.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct BsdInfo {
    #[deku(endian = "big")]
    owner_id: u32,
    #[deku(endian = "big")]
    group_id: u32,
    admin_flags: u8,
    owner_flags: u8,
    #[deku(endian = "big")]
    file_mode: u16,
    special: BsdInfoSpecial,
}

// TODO Populate fileMode enum once it needs to be referenced
#[repr(u32)]
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "big", type = "u32"))]
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
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct ExtentDescriptor {
    pub start_block: u32,
    pub block_count: u32,
}

/// When an extent descriptor is not used, it is set to zero.
const UNUSED_EXTENT_DESCRIPTOR: ExtentDescriptor = ExtentDescriptor {
    start_block: 0,
    block_count: 0,
};

/// A file's extent record is 8 Extent Descriptors
// TODO Convert to Option<ExtentDescriptor>
pub type ExtentRecord = [ExtentDescriptor; 8];

/// Resource and Data Fork contents. Defined as `struct HFSPlusForkData` in
/// TN1150 > Fork Data Structure.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct ForkData {
    pub logical_size: u64,
    pub clump_size: u32,
    pub total_blocks: u32,

    pub extents: ExtentRecord,
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
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct VolumeHeader {
    #[deku(endian = "big")]
    pub signature: u16,
    #[deku(endian = "big")]
    pub version: u16,
    #[deku(endian = "big")]
    pub attributes: u32,
    #[deku(endian = "big")]
    pub last_mounted_version: u32,
    #[deku(endian = "big")]
    pub journal_info_block: u32,

    #[deku(endian = "big")]
    pub create_date: Date,
    #[deku(endian = "big")]
    pub modify_date: Date,
    #[deku(endian = "big")]
    pub backup_date: Date,
    #[deku(endian = "big")]
    pub checked_date: Date,

    #[deku(endian = "big")]
    pub file_count: u32,
    #[deku(endian = "big")]
    pub folder_count: u32,

    #[deku(endian = "big")]
    pub block_size: u32,
    #[deku(endian = "big")]
    pub total_blocks: u32,
    #[deku(endian = "big")]
    pub free_blocks: u32,

    #[deku(endian = "big")]
    pub next_allocation: u32,
    #[deku(endian = "big")]
    pub rsrc_clump_size: u32,
    #[deku(endian = "big")]
    pub data_clump_size: u32,
    #[deku(endian = "big")]
    pub next_catalog_id: CatalogNodeId,

    #[deku(endian = "big")]
    pub write_count: u32,
    #[deku(endian = "big")]
    pub encodings_bitmap: u64,

    #[deku(endian = "big")]
    pub finder_info: [u32; 8],

    pub allocation_file: ForkData,
    pub extents_file: ForkData,
    pub catalog_file: ForkData,
    pub attributes_file: ForkData,
    pub startup_file: ForkData,
}

/// Catalog Node ID or CNID identifies a B-tree file.
/// Defined in TN1150 > Catalog File.
pub type CatalogNodeId = u32;

#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u32)]
pub enum StandardCnid {
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
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct BTreeNodeDescriptor {
    #[deku(endian = "big")]
    pub forward_link: u32,
    #[deku(endian = "big")]
    pub backward_link: u32,
    pub kind: BTreeNodeKind,
    pub height: u8,
    #[deku(endian = "big")]
    pub num_records: u16,
    #[deku(endian = "big")]
    pub reserved: u16,
}

impl BTreeNodeDescriptor {
    pub const SIZE: usize = 14;
}

/// Known values for BTreeNodeDescriptor::kind.
/// Defined in docs for `struct BTNodeDescriptor` in TN1150 > Catalog File.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(
    feature = "deku",
    deku(endian = "endian", ctx = "endian: Endian", type = "i8")
)]
#[repr(i8)]
pub enum BTreeNodeKind {
    kBTLeafNode = -1,
    kBTIndexNode = 0,
    kBTHeaderNode = 1,
    kBTMapNode = 2,
}

/// BTree Header describing upcoming BTree Structure.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct BTreeHeaderRecord {
    #[deku(endian = "big")]
    pub tree_depth: u16,
    #[deku(endian = "big")]
    pub root_node: u32,
    #[deku(endian = "big")]
    pub leaf_records: u32,
    #[deku(endian = "big")]
    pub first_leaf_node: u32,
    #[deku(endian = "big")]
    pub last_leaf_node: u32,
    #[deku(endian = "big")]
    pub node_size: u16,
    #[deku(endian = "big")]
    pub max_key_length: u16,
    #[deku(endian = "big")]
    pub total_nodes: u32,
    #[deku(endian = "big")]
    pub free_nodes: u32,
    #[deku(endian = "big")]
    pub reserved_1: u16,
    #[deku(endian = "big")]
    pub clump_size: u32,
    pub btree_type: BTreeType,
    pub key_compare_type: BTreeKeyCompareType,
    #[deku(endian = "big")]
    pub attributes: u32,
    #[deku(endian = "big")]
    pub reserved_3: [u32; 16],
}

impl BTreeHeaderRecord {
    pub const SIZE: usize = 106;
}

#[derive(Debug, DekuRead)]
pub struct BTreeUserDataRecord {
    reserved: [u8; 128],
}

impl BTreeUserDataRecord {
    pub const SIZE: usize = 128;
}

pub type BTreeKey = Vec<u8>;

// Can we specify size for Deku parse?
pub struct BTreeAllocationMapRecord {
    bitmap: Vec<u8>,
}

impl BTreeAllocationMapRecord {
    // Algorithm taken from IsAllocationBlockUsed in  TN1150 > Allocation File
    fn isBlockUsed(&self, allocation_block: u32) -> bool {
        // TODO handle overflow?
        let offset = allocation_block / 8;
        let this_byte = self.bitmap[offset as usize];
        let bit_mask = 1 << (7 - (allocation_block & 8));

        let is_set = this_byte & bit_mask != 0;

        is_set
    }
}

/// Information about a catalog file.
/// Defined as `struct HFSPlusCatalogKey` in TN1150 > Catalog File.
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct CatalogFileKey {
    #[deku(endian = "big")]
    pub length: u16,
    #[deku(endian = "big")]
    pub parent: CatalogNodeId,
    pub name: HFSUniStr255,
}

// TODO Deku chould handle parsing CatalogFileKey from bytes.
impl TryFrom<Vec<u8>> for CatalogFileKey {
    type Error = io::Error;

    /// Read Key from a series of bytes. Key Length is implicit.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut cur = Cursor::new(&value);

        // Length is derived from input slice
        let length = value.len() as u16;

        // Key Data
        let mut key: BTreeKey = vec![0u8; length as usize];
        cur.read_exact(&mut key)?;

        let mut key_cur = Cursor::new(&key);

        // Key: Parent CNID
        let mut buf = [0u8; 4];
        key_cur.read_exact(&mut buf)?;
        let parent = CatalogNodeId::from_be_bytes(buf);

        // Key: String Length
        let mut buf = [0u8; 2];
        key_cur.read_exact(&mut buf)?;
        let char_count = u16::from_be_bytes(buf) as usize;

        // Key: File Name
        let mut string = vec![0u16; 255];
        for i in 0..char_count {
            let mut buf = [0u8; 2];
            key_cur.read_exact(&mut buf)?;
            let char = u16::from_be_bytes(buf);
            string[i] = char;
        }

        let name = HFSUniStr255 {
            #[cfg(not(feature = "deku"))]
            length: char_count as u16,
            unicode: string,
        };

        Ok(Self {
            length,
            parent,
            name,
        })
    }
}

// TODO Deku should handle serializing CatalogFileKey to bytes.
impl Into<Vec<u8>> for CatalogFileKey {
    fn into(self) -> Vec<u8> {
        let len =  4 // Parent CNID (u32) 
            + 2 // Name Length (u16) 
            + 2 * self.name.unicode.len() // Bytes
            ;

        let mut out = Vec::<u8>::with_capacity(len as usize);
        out.extend_from_slice(self.parent.to_be_bytes().as_slice());
        out.extend_from_slice((self.name.unicode.len() as u16).to_be_bytes().as_slice());
        self.name
            .unicode
            .iter()
            .for_each(|c16| out.extend_from_slice(c16.to_be_bytes().as_slice()));

        out
    }
}

/// Type of data contained in this catalog file.
/// Defined in documentation for `struct HFSPlusCatalogKey` in
/// TN1150 > Catalog File Data.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(
    feature = "deku",
    deku(endian = "endian", ctx = "endian: Endian", type = "u16")
)]
#[repr(u16)]
pub enum CatalogFileDataType {
    kHFSPlusFolderRecord = 0x0001,
    kHFSPlusFileRecord = 0x0002,
    kHFSPlusFolderThreadRecord = 0x0003,
    kHFSPlusFileThreadRecord = 0x0004,
}

/// Helper definitions for inspecting legacy HFS, which used
/// one byte to store the record type followed by a reserved byte.
/// When parsing a legacy HFS volume, the endianness will be
/// switched as a result.
/// Defined in documentation for `struct HFSPlusCatalogKey` in
/// TN1150 > Catalog File Data.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[deprecated]
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
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(
    feature = "deku",
    deku(endian = "endian", ctx = "endian: Endian", type = "u8")
)]
#[repr(u8)]
pub enum BTreeType {
    kHFSBTreeType = 0,    // control file
    kUserBTreeType = 128, // user btree type starts from 128
    kReservedBTreeType = 255,
}

// Comparison mode, depending on HFSX support.
/// Defined in documentation for `struct HFSPlusCatalogKey` in
/// TN1150 > Catalog File Data.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(
    feature = "deku",
    deku(endian = "endian", ctx = "endian: Endian", type = "u8")
)]
#[repr(u8)]
pub enum BTreeKeyCompareType {
    reserved_hfsx_only = 0x00,
    kHFSCaseFolding = 0xCF,
    kHFSBinaryCompare = 0xBC,
}

// TODO User Data Record: Second record in header node must be 128 bytes.

// TODO Map Record: Occupies remaining space in header node.

/// BTree leaf node for Folders. Defined as `struct HFSPlusCatalogFolder`
/// in TN1150 > Catalog Folder Records
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct CatalogFolder {
    /// Always CatalogFolderDataType::kHFSPlusFolderRecord
    pub record_type: CatalogFileDataType,
    pub flags: u16,
    pub valence: u32,
    pub folder_id: CatalogNodeId,
    pub create_date: Date,
    pub content_mod_date: Date,
    pub attribute_mod_date: Date,
    pub access_date: Date,
    pub backup_date: Date,
    pub permissions: BsdInfo,
    pub user_info: FolderInfo,
    pub finder_info: ExtendedFolderInfo,
    pub text_encoding: u32, // TextEncoding,
    pub reserved: u32,
}

pub enum CatalogLeafRecord {
    Folder(CatalogFolder),
    File(CatalogFile),
    FolderThread(CatalogThread),
    FileThread(CatalogThread),
}

/// Defined in documentation for `struct HFSPlusCatalogFile` in
/// TN1150 > Catalog File Records.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u16)]
enum CatalogFileBit {
    kHFSFileLockedBit = 0x0000,
    kHFSThreadExistsBit = 0x0001,
}

/// Defined in documentation for `struct HFSPlusCatalogFile` in
/// TN1150 > Catalog File Records.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u16)]
enum CatalogFileBitMask {
    kHFSFileLockedMask = 0x0001,
    kHFSThreadExistsMask = 0x0002,
}

/// BTree leaf node for Files. Defined as `struct HFSPlusCatalogFile` in
/// TN1150 > Catalog File Records
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(
    feature = "deku",
    deku(endian = "endian", ctx = "endian: Endian", ctx_default = "Endian::Big")
)]
pub struct CatalogFile {
    pub record_type: CatalogFileDataType,
    pub flags: u16,
    pub reserved_1: u32,
    pub file_id: CatalogNodeId,
    pub create_date: Date,
    pub content_mod_date: Date,
    pub attribute_mod_date: Date,
    pub access_date: Date,
    pub backup_date: Date,
    pub permissions: BsdInfo,
    pub user_info: FileInfo,
    pub finder_info: ExtendedFileInfo,
    pub text_encoding: u32, // TextEncoding,
    pub reserved_2: u32,

    pub data_fork: ForkData,
    pub resource_fork: ForkData,
}

/// BTree link to CNID. Defined as `struct HFSPlusCatalogThread` in
/// TN1150 > Catalog Thread Records.
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(
    feature = "deku",
    deku(endian = "endian", ctx = "endian: Endian", ctx_default = "Endian::Big")
)]
pub struct CatalogThread {
    pub record_type: CatalogFileDataType,
    #[deku(endian = "big")]
    pub reserved: i16,
    #[deku(endian = "big")]
    pub parent_id: CatalogNodeId,
    pub node_name: HFSUniStr255,
}

/// A location on screen, used to store window placement.
/// Defined in TN1150 > Finder Info.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(
    feature = "deku",
    deku(endian = "endian", ctx = "endian: Endian", ctx_default = "Endian::Big")
)]
struct Point {
    v: i16,
    h: i16,
}

/// Rectangular region used for Directory windows.
/// Defined in TN1150 > Finder Info.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
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
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct FileInfo {
    #[deku(endian = "big")]
    file_type: OSType,
    #[deku(endian = "big")]
    file_creator: OSType,
    #[deku(endian = "big")]
    finder_flags: u16,
    location: Point,
    #[deku(endian = "big")]
    reserved: u16,
}

/// Additional file information for display in Finder
/// Defined in TN1150 > Finder Info.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct ExtendedFileInfo {
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
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct FolderInfo {
    window_bounds: Rect,
    finder_flags: u16,
    location: Point,
    reserved: u16,
}

/// Finder Metadata and display information
/// Defined in TN1150 > Finder Info.
#[derive(Debug)]
#[cfg_attr(feature = "deku", derive(DekuRead))]
#[cfg_attr(feature = "deku", deku(endian = "endian", ctx = "endian: Endian"))]
pub struct ExtendedFolderInfo {
    scroll_position: Point,
    reserved_1: i32,
    extended_finder_flags: u16,
    reserved_2: i16,
    put_away_folder_id: i32,
}

/// Defined as `struct HFSPlusExtentKey` in TN1150 > Extents Overflow File
/// Key.
struct ExtentKey {
    key_length: u16,
    fork_type: ExtentKeyForkType,
    pad: u8,
    file_id: CatalogNodeId,
    start_block: u32,
}

/// Defined in docs for struct HFSPlusExtentKey` in
/// TN1150 > Extents Overflow File Key.
#[repr(u8)]
enum ExtentKeyForkType {
    Data = 0x00,
    Resource = 0xFF,
}

/// Defined in TN1150 > Attributes File Data
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u32)]
enum AttributeForkDataType {
    kHFSPlusAttrInlineData = 0x10,
    kHFSPlusAttrForkData = 0x20,
    kHFSPlusAttrExtents = 0x30,
}

/// Defined as `struct HFSPlusAttrForkData` in TN1150 > Fork Data Attributes.
struct AttributeForkData {
    record_type: AttributeForkDataType,
    reserved: u32,
    fork: ForkData,
}

/// Defined as `struct HFSPlusATtrExtents` in TN1150 > Extension Attributes.
struct AttributeExtents {
    record_type: AttributeForkDataType,
    reserved: u32,
    extents: ExtentRecord,
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
