#![deny(dead_code, unsafe_code)]

#[repr(u16)]
pub enum VolumeSignature {
    HfsPlus = u16::from_be_bytes(*b"H+"),
    HfsX = u16::from_be_bytes(*b"HX"),
}

#[repr(u16)]
pub enum VolumeVersion {
    HfsPlus = 4,
    HfsX = 5
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

/// Volume header, offset 1024 bytes from start of disk.
///
/// Described in TN1150 [Volume Header](https://developer.apple.com/library/archive/technotes/tn/tn1150.html#VolumeHeader)
#[repr(C)]
pub struct VolumeHeader {
    pub signature: VolumeSignature,
    pub version: VolumeVersion,
    pub attributes: u32,
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
