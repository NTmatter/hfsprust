#![deny(dead_code, unsafe_code)]

use std::num::NonZeroU32;

#[repr(C)]
pub struct UnicodeString255 {
    pub length: u16,
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
#[repr(C)]
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

/// Information about the size and location of a file.
///
/// Described in TN1150 [Fork Data Structure](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ForkDataStructure)
#[repr(C)]
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
    pub extents: [ExtentDescriptor; 8],
}

/// Identifies the start and length (in blocks) of an extent.
///
/// Described in TN1150 [Fork Data Structure](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#ForkDataStructure)
#[repr(C)]
pub struct ExtentDescriptor {
    pub start_block: u32,
    pub block_count: u32,
}

/// File ownership, permissions, mode, and type-specific information.
///
/// The meaning of the `special` field depends on the context in which the
/// descriptor is being used.
///
/// Described in TN1150 [HFS Plus Permissions](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HFSPlusPermissions)
#[repr(C)]
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
#[repr(C)]
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

/// Described in TN1150 [B-Trees](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#BTrees)
#[repr(C)]
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

/// B-tree file header.
///
/// Described in TN1150 [Header Record](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#HeaderRecord)
#[repr(packed)]
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

#[repr(C)]
pub struct CatalogKey {
    pub key_length: u16,
    pub parent_id: CatalogNodeId,
    pub node_name: UnicodeString255,
}
