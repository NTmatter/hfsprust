// SPDX-License-Identifier: MIT

//! Types and constants from Apple's [TN1150 - HFS Plus Volume Format](https://developer.apple.com/library/archive/technotes/tn/tn1150.html),
//! adjusted to use Rust-friendly naming.

#![forbid(dead_code, unsafe_code, unused)]

use std::num::NonZeroU32;

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct UnicodeString255 {
    pub length: u16,
    // DESIGN Should this be a Vec<u8>, or is it a fixed-size array?
    pub unicode: [u16; 255],
}

#[repr(u16)]
pub enum VolumeSignature {
    HfsPlus = u16::from_be_bytes(*b"H+"),
    HfsX = u16::from_be_bytes(*b"HX"),
}

#[repr(u16)]
pub enum VolumeVersion {
    HfsPlus = 4,
    HfsX = 5,
}

// DESIGN Would this be better as a NewType `DateTime(u32)`?
/// Represents seconds since 01-01-1904 GMT, including a leap day for years evenly
/// divisible by four.
///
/// Exception for [`field@VolumeHeader::creation_date`],
/// which is stored in local time.
///
/// Described in TN1150 [HFS Plus Dates](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HFSPlusDates).
pub type DateTime = u32;

/// Catalog Node ID
///
/// Described in TN1150 [Catalog File](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFile)
pub type CatalogNodeId = u32;

#[repr(u32)]
pub enum SpecialFileCatalogNodeId {
    RootParent = 1,
    RootFolder = 2,
    ExtentsFile = 3,
    CatalogFile = 4,
    BadBlockFile = 5,
    AllocationFile = 6,
    StartupFile = 7,
    AttributesFile = 8,
    RepairCatalogFile = 14,
    BogusExtentFile = 15,
    FirstUserCatalogNode = 16,
}

#[repr(u32)]
pub enum KnownCreatorCodes {
    MacOs81 = u32::from_be_bytes(*b"8.10"),
    MacOsX = u32::from_be_bytes(*b"10.0"),
    MacOsXJournaled = u32::from_be_bytes(*b"HFSJ"),
    FsckHfs = u32::from_be_bytes(*b"fsck"),
}

/// Volume header, offset 1024 bytes from start of disk.
///
/// Described in TN1150 [Volume Header](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#VolumeHeader)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct VolumeHeader {
    pub signature: VolumeSignature,
    pub version: VolumeVersion,
    pub attributes: u32,

    /// 4-character Creator Code of the last version to mount the volume
    pub last_mounted_version: u32,
    pub journal_info_block: u32,

    pub creation_date: DateTime,
    pub modify_date: DateTime,
    pub backup_date: DateTime,
    pub checked_date: DateTime,

    pub file_count: u32,
    pub folder_count: u32,

    pub block_size: u32,
    pub total_blocks: u32,
    pub free_blocks: u32,

    pub next_allocation: u32,
    pub resource_fork_clump_size: u32,
    pub data_clump_size: u32,
    pub next_catalog_id: CatalogNodeId,

    pub write_count: u32,
    pub encodings_bitmap: u64,

    // TODO Convert to struct, as they have names
    pub finder_info: [u32; 8],

    pub allocation_file: ForkData,
    pub extents_file: ForkData,
    pub catalog_file: ForkData,
    pub attributes_file: ForkData,
    pub startup_file: ForkData,
}

#[repr(u32)]
pub enum VolumeAttributeMask {
    // Bits 0-7 are reserved. Note that macOS uses bit 7 to indicate hardware
    // read-only status.
    /// Volume is write-protected due to hardware setting (macOS only).
    ///
    /// This may indicate that hardware is preventing writes to this volume.
    HardwareLock = 1 << 7,

    /// The volume was correctly flushed before being unmounted. This bit must
    /// be cleared when the volume is mounted for writing. If it is set when
    /// mounting, a consistency check is necessary.
    Unmounted = 1 << 8,

    /// The overflow file contains bad block records.
    SparedBlocks = 1 << 9,

    /// Blocks from this volume should not be cached.
    NoCacheRequired = 1 << 10,

    /// The volume is currently mounted, inverted from the Unmounted bit. If this
    /// bit is set, the volume requires a consistency check.
    VolumeInconsistent = 1 << 11,

    /// The `next_catalog_id` field has overflowed, requiring reuse of smaller ids.
    CatalogNodeIdsReused = 1 << 12,

    /// The volume is journaled, and the journal is available at the journal_info_block
    VolumeJournaled = 1 << 13,

    // Bit 14 is reserved
    /// Volume is write-protected by software. Any implementation MUST refuse to
    /// write if this bit is set.
    SoftwareLock = 1 << 15,
    // Bits 16-31 are reserved
}

/// Information about the size and location of a file.
///
/// Described in TN1150 [Fork Data Structure](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ForkDataStructure)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct ForkData {
    /// Total size of data, in bytes
    pub logical_size: u64,

    /// Per-file clump size when used in Volume Header, total blocks read for hot files
    /// when used in Catalog Record.
    ///
    /// See [Hot Files](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HotFile)
    pub clump_size: u32,

    /// Total number of blocks allocated for all extents in fork
    pub total_blocks: u32,

    /// First eight extent descriptors. Any remaining descriptors are stored in
    /// the [Extents Overflow File](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ExtentsOverflowFile).
    pub extents: ExtentRecord,
}

/// Identifies the start and length (in blocks) of an extent.
///
/// Described in TN1150 [Fork Data Structure](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ForkDataStructure)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct ExtentDescriptor {
    pub start_block: u32,
    pub block_count: u32,
}

pub type ExtentRecord = [ExtentDescriptor; 8];

/// File ownership, permissions, mode, and type-specific information.
///
/// The meaning of the `special` field depends on the context in which the
/// descriptor is being used.
///
/// Described in TN1150 [HFS Plus Permissions](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HFSPlusPermissions)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct BsdInfo {
    pub owner_id: u32,
    pub group_id: u32,
    pub admin_flags: u8,
    pub owner_flags: u8,
    pub file_mode: u16,

    /// Context-specific reference count (for hard links), number of hard links
    /// (for indirect node), or device id (for raw devices).
    ///
    /// TN1150 defines this field as a [union](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#Union),
    /// but this implementation uses a single field to avoid unsafe access to
    /// union fields.
    #[cfg(not(feature = "file_info_union"))]
    pub special: u32,

    #[cfg(feature = "file_info_union")]
    pub special: HFSPlusBSDInfoSpecial,
}

#[cfg(feature = "file_info_union")]
#[cfg_attr(feature = "repr_c", repr(C))]
pub union HFSPlusBSDInfoSpecial {
    pub link_reference_count: u32,
    pub hardlink_count: u32,
    pub raw_device_number: u32,
}

#[repr(u8)]
pub enum BsdInfoAdminFlags {
    Archived = 1,
    Immutable = 2,
    AppendOnly = 4,
}

#[repr(u8)]
pub enum BsdInfoOwnerFlags {
    NoDump = 1,
    Immutable = 2,
    AppendOnly = 4,
    Opaque = 8,
}

#[repr(u16)]
pub enum BsdInfoFileModeFlag {
    SetUid = 0o00_4000,
    SetGid = 0o00_2000,
    Sticky = 0o00_1000,

    OwnerRwxMask = 0o00_0700,
    OwnerRead = 0o00_0400,
    OwnerWrite = 0o00_0200,
    OwnerExecute = 0o00_0100,

    GroupRwxMask = 0o00_0070,
    GroupRead = 0o00_0040,
    GroupWrite = 0o00_0020,
    GroupExecute = 0o00_0010,

    OtherRwxMask = 0o00_0007,
    OtherRead = 0o00_0004,
    OtherWrite = 0o00_0002,
    OtherExecute = 0o00_0001,

    FileTypeMask = 0o17_0000,
    NamedPipe = 0o01_0000,
    CharacterSpecial = 0o02_0000,
    Directory = 0o04_0000,
    BlockSpecial = 0o06_0000,
    Regular = 0o10_0000,
    SymbolicLink = 0o12_0000,
    Socket = 0o14_0000,
    Whiteout = 0o16_0000,
}

// region B-tree

/// Described in TN1150 [B-Trees](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#BTrees)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct BTreeNodeDescriptor {
    /// Node Number of the next node of this type, or none if this is the last node.
    pub forward_link: Option<NonZeroU32>,
    /// Node Number of the previous node of this type, or none if this is the first node.
    pub back_link: Option<NonZeroU32>,
    /// Type of this node
    pub kind: BTreeNodeType,
    /// Depth of this node in the BTree Hierarchy. Must be zero for the root node.
    pub height: u8,
    /// Number of records contained by this node.
    pub record_count: u16,
    /// Reserved field.
    pub reserved: u16,
}

/// Described in TN1150 [B-Trees](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#BTrees)
#[repr(i8)]
pub enum BTreeNodeType {
    /// Data Record
    Leaf = -1,
    /// Pointer Record
    Index = 0,
    /// Header Record
    Header = 1,
    /// Map Record
    Map = 2,
}

// TODO Conditional packed representation for memory transmute.
/// B-tree file header.
///
/// Uses `repr(packed)` to handle misaligned `clump_size` and `attributes` fields.
///
/// Described in TN1150 [Header Record](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HeaderRecord)
#[cfg_attr(all(feature = "repr_c", not(feature = "packed_btree")), repr(C))]
#[cfg_attr(all(not(feature = "repr_c"), feature = "packed_btree"), repr(packed))]
#[cfg_attr(all(feature = "repr_c", feature = "packed_btree"), repr(C, packed))]
pub struct BTreeHeaderRecord {
    /// Current depth of the tree. This should be equal to the Root Node's height.
    pub tree_depth: u16,

    /// Node Number of BTree's root node
    pub root_node: u32,

    /// Total number of leaf records contained in all leaf nodes.
    pub leaf_records: u32,

    /// Node Number of first leaf node, if there are any leaf nodes.
    pub first_leaf_node: Option<NonZeroU32>,

    /// Node Number of last leaf node, if there are any leaf nodes.
    pub last_leaf_node: Option<NonZeroU32>,

    /// Size of node in bytes.
    ///
    /// Must be a power of 2, from 512 to 32,768 inclusive.
    pub node_size: u16,

    // TODO Extract key lengths from HFSVolumes.h.
    /// Maximum length of key in an index or leaf node.
    pub max_key_length: u16,

    /// Total number of nodes (free or used) in the B-tree.
    pub total_nodes: u32,

    /// Number of unused nodes in the B-tree.
    pub free_nodes: u32,

    pub reserved_1: u16,

    /// Misaligned. Ignored in HFS+, should be treated as Reserved.
    pub clump_size: u32,

    pub btree_type: BTreeType,

    /// Case Sensitivity for HFSX volumes. Treat as reserved for non-HFSX volumes.
    pub key_compare_type: BTreeKeyCompareType,

    /// Misaligned list of volume attributes.
    pub attributes: u32,

    /// Reserved.
    pub reserved_3: [u32; 16],
}

/// Identifier for this B-tree's type.
///
/// Described in TN1150 [Header Record](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HeaderRecord)
#[repr(u8)]
pub enum BTreeType {
    /// Control File. Catalog, extent, and attribute trees.
    Hfs = 0,

    // 1-127 used in macOS 9 and earlier.
    /// User BTree
    User = 128,

    /// Reserved in modern HFS+. Formerly used in macOS 9 and earlier.
    Reserved = 255,
}

/// Case sensitivity for keys in this B-tree.
///
/// Described in TN1150 [Header Record](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HeaderRecord)
#[repr(u8)]
pub enum BTreeKeyCompareType {
    /// Case-insensitive comparisons
    CaseFolding = 0xCF,
    /// Binary comparison
    BinaryCompare = 0xBC,
}

/// Described in TN1150 [Header Record](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HeaderRecord)
#[repr(u32)]
pub enum BTreeAttributeMask {
    /// BTree was not closed properly, and needs to be checked for consistency. Not used in HFS+.
    BadClose = 1,

    /// When true, key length must be u16, otherwise u8. Must be set for all HFS+ B-trees.
    BigKeys = 2,

    /// Keys occupy the number of bytes indicated by their key length field, otherwise maxKeyLength bytes.
    /// Must be set for the HFS+ Catalog B-tree, and cleared for the HFS+ Extents B-tree.
    VariableIndexKeys = 4,
}

#[cfg_attr(feature = "repr_c", repr(C))]
pub struct UserDataRecord(pub [u8; 128]);

/// Allocation File Bitmap
///
/// Described by TN1150 in [Allocation File](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#AllocationFile)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct AllocationMapRecord(pub Vec<u8>);

impl AllocationMapRecord {
    /// Determine if a block has been set in the bitmap.
    ///
    /// Returns whether the bit is set, or an error if block index is out of bounds.
    pub fn is_block_used(&self, block: u32) -> Result<bool, ()> {
        let offset = block / 8;
        let Some(byte) = self.0.get(offset as usize) else {
            return Err(());
        };

        let bit_offset = block % 8;
        let mask = (1 << (7 - bit_offset)) as u8;

        let is_set = byte & mask != 0;

        Ok(is_set)
    }
}

// endregion

/// Described by TN1150 in [Catalog File Key](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFile)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct CatalogKey {
    pub length: u16,
    pub parent_id: CatalogNodeId,
    pub node_name: UnicodeString255,
}

/// Type of data contained in this catalog file.
///
/// Described by TN1150 in [Catalog File Data](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFile)
#[repr(i16)]
pub enum CatalogFileDataRecordType {
    /// BTree Record Type for a folder, to be interpreted as a `CatalogFolder`.
    Folder = 0x0001,
    File = 0x0002,
    FolderThread = 0x0003,
    FileThread = 0x0004,
}

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
pub type OsType = FourCharCode;

/// B-tree record holding information about a folder.
///
/// Described by TN1150 in [Catalog Folder Records](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFolderRecord)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct CatalogFolder {
    /// Should always be `CatalogFileDataRecordType::Folder`.
    pub record_type: CatalogFileDataRecordType,

    /// No flags are currently defined. Treat as a reserved field.
    pub flags: u16,

    /// Number of files and folders directly contained by this folder.
    ///
    pub valence: u32,
    /// ID of parent folder
    pub folder_id: CatalogNodeId,
    pub create_date: DateTime,
    pub content_modification_date: DateTime,
    pub attribute_modification_date: DateTime,
    pub access_date: DateTime,
    pub backup_date: DateTime,
    pub permissions: BsdInfo,
    pub user_info: FolderInfo,
    pub finder_info: ExtendedFolderInfo,
    pub text_encoding: u32,
    pub reserved: u32,
}

/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct FolderInfo {
    pub window_bounds: Rect,
    pub finder_flags: u16,
    pub location: Point,
    pub reserved_field: u16,
}

/// Type of data contained in this catalog file (legacy HFS only)
///
/// Described by TN1150 in [Catalog File Data](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFile)
#[repr(u16)]
pub enum CatalogFileDataThreadRecordType {
    Folder = 0x0100,
    File = 0x0200,
    FolderThread = 0x0300,
    FileThread = 0x0400,
}

/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct ExtendedFolderInfo {
    pub scroll_position: Point,
    pub reserved_1: i32,
    pub extended_finder_flags: u16,
    pub reserved_2: i16,
    pub put_away_folder_id: u32,
}

/// B-tree record holding information about a file on the volume.
///
/// Described by TN1150 in [Catalog File Records](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogFileRecord)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct CatalogFile {
    /// Should always be `CatalogFileDataRecordType::File`.
    pub record_type: CatalogFileDataRecordType,

    /// A bitfield of `CatalogFileFlag`
    pub flags: u16,
    pub reserved_1: u32,
    pub file_id: CatalogNodeId,
    pub create_date: DateTime,
    pub content_modification_date: DateTime,
    pub attribute_modification_date: DateTime,
    pub access_date: DateTime,
    pub backup_date: DateTime,
    pub permissions: BsdInfo,
    pub user_info: FileInfo,
    pub finder_info: ExtendedFileInfo,
    pub text_encoding: u32,
    pub reserved_2: u32,

    pub data_fork: ForkData,
    pub resource_fork: ForkData,
}

#[repr(u16)]
pub enum CatalogFileFlag {
    /// None of the forks may be modified, but they may be opened for reading.
    ///
    /// Catalog information may still be changed.
    FileLocked = 1,
    ThreadExists = 2,
}

/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct FileInfo {
    pub file_type: OsType,
    pub file_creator: OsType,
    pub finder_flags: u16,
    /// Coordinate relative to parent folder
    pub location: Point,
    pub reserved: u16,
}

#[repr(u32)]
pub enum WellKnownFileTypeCode {
    /// File type code for Hardlink files
    ///
    /// Described by TN1150 in [Hard Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HardLinks)
    HardLink = u32::from_be_bytes(*b"hlnk"),

    /// File type code for Symlink files
    ///
    /// Described by TN1150 in [Symbolic Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#Symlinks)
    SymbolicLink = u32::from_be_bytes(*b"slnk"),
}

#[repr(u32)]
pub enum WellKnownFileCreatorCode {
    /// Creator code for Hardlink files
    ///
    /// Described by TN1150 in [Hard Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HardLinks)
    HardLink = u32::from_be_bytes(*b"hfs+"),

    /// Creator code for Symlink files
    ///
    /// Described by TN1150 in [Symbolic Links](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#Symlinks)
    SymbolicLink = u32::from_be_bytes(*b"rhap"),
}

/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct ExtendedFileInfo {
    pub reserved_1: [i16; 4],
    pub extended_finder_flags: u16,
    pub reserved_2: i16,
    pub put_away_folder_id: i32,
}

/// B-tree record linking a Catalog Node ID to a file.
///
/// Described by TN1150 in [Catalog Thread Records](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#CatalogThreadRecord)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct CatalogThread {
    /// One of `CatalogFileDataRecordType::FolderThread` or `CatalogFileDataRecordType::FileThread`.
    ///
    /// NB: TN1150 contradicts itself; Catalog File Data specifies that this should be
    /// `FolderThread` or `FileThread`, whereas Catalog Thread Records specifies that this should be
    /// `Folder` or `File` instead. Contextually, the `*Thread` variants make more sense here, as
    /// the `Folder` and `File` variants are consumed by `CatalogFolder` and `CatalogFile`.
    pub record_type: CatalogFileDataRecordType,
    pub reserved: i16,

    /// Parent CNID of the File or Folder referenced by this record.
    pub parent_id: CatalogNodeId,

    /// Name of the file or folder referenced yb this record.
    pub node_name: UnicodeString255,
}

/// Finder flags (finderFlags, fdFlags and frFlags)
///
/// Described by TN1150 in [Finder Info](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#FinderInfo)
#[repr(u16)]
pub enum FinderFlags {
    /// Files and folders (System 6)
    IsOnDesk = 0x0001,

    /// Files and folders
    Color = 0x000E,

    /// Files only (Applications only) If clear, the application needs to write to its resource fork,
    /// and therefore cannot be shared on a server
    IsShared = 0x0040,

    /// This file contains no INIT resource. Files only (Extensions/Control Panels only)
    HasNoInits = 0x0080,

    /// Files only.  Clear if the file contains desktop database resources ('BNDL', 'FREF', 'open',
    /// 'kind'...) that have not been added yet.  Set only by the Finder. Reserved for folders
    HasBeenInited = 0x0100,

    /// Files and folders
    HasCustomIcon = 0x0400,

    /// Files only
    IsStationery = 0x0800,

    /// Files and folders
    NameLocked = 0x1000,

    /// Files only
    HasBundle = 0x2000,

    /// Files and folders
    IsInvisible = 0x4000,

    /// Files only
    IsAlias = 0x8000,
}

#[repr(u16)]
pub enum ExtendedFinderFlags {
    /// The other extended flags should be ignored
    FlagsAreInvalid = 0x8000,

    /// The file or folder has a badge resource
    HasCustomBadge = 0x0100,

    /// The file contains routing info resource
    HasRoutingInfo = 0x0004,
}

/// Extents Overflow File Key
///
/// Described by TN1150 in [Extents Overflow File Key](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ExtentsOverflowFile)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct ExtentKey {
    pub key_length: u16,
    /// Type of fork for which this record applies. Must be 0x00 for the Data Fork
    /// and 0xFF for the resource fork.
    pub fork_type: ExtentKeyForkType,
    pub padding: u8,
    pub file_id: CatalogNodeId,
    pub start_block: u32,
}

#[repr(u8)]
pub enum ExtentKeyForkType {
    Data = 0x00,
    Resource = 0xFF,
}

/// Described by TN1150 in [Fork Data Attributes](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ForkDataAttributes]
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct AttributeForkData {
    pub record_type: AttributeForkDataRecordType,
    pub reserved: u32,
    pub fork_data: ForkData,
}

/// Described by TN1150 in [Extension Attributes](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ExtensionAttributes)
#[cfg_attr(feature = "repr_c", repr(C))]
pub struct AttributeExtents {
    pub record_type: AttributeForkDataRecordType,
    pub reserved: u32,
    pub extents: ExtentRecord,
}

#[repr(u32)]
pub enum AttributeForkDataRecordType {
    /// Reserved for future use.
    InlineData = 0x10,

    /// The record is a Fork Data Attribute. It should be interpreted as an `AttributeForkData`
    ForkData = 0x20,

    /// The record is an extension attribute. It should be interpreted as an `AttributeExtent`
    Extents = 0x30,
}
