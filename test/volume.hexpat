
using HFSCatalogNodeID = be u32;

struct HFSPlusExtentDescriptor {
  be u32 startBlock;
  be u32 blockCount;
};


struct HFSPlusForkData {
  be u64 logicalSize;
  be u32 clumpSize;
  be u32 totalBlocks;
  HFSPlusExtentDescriptor extents[8];
};

struct HFSPlusVolumeHeader {
  be u16 signature;
  be u16 version;
  be u32 attributes;
  be u32 lastMountedVersion;
  be u32 journalInfoBlock;

  be u32 createDate;
  be u32 modifyDate;
  be u32 backupDate;
  be u32 checkedDate;

  be u32 fileCount;
  be u32 folderCount;

  be u32 blockSize;
  be u32 totalBlocks;
  be u32 freeBlocks;

  be u32 nextAllocation;
  be u32 rsrcClumpSize;
  be u32 dataClumpSize;
  HFSCatalogNodeID nextCatalogID;

  be u32 writeCount;
  be u64 encodingsBitmap;

  be u32 finderInfo[8];

  HFSPlusForkData allocationFile;
  HFSPlusForkData extentsFile;
  HFSPlusForkData catalogFile;
  HFSPlusForkData attributesFile;
  HFSPlusForkData startupFile;

};

struct JournalInfoBlock {
  be u32 flags;
  be u32 device_signature[8];
  be u64 offset;
  be u64 size;
  be u32 reserved[32];
};

struct BTNodeDescriptor {
  be u32 fLink;
  be u32 bLink;
  u8 kind;
  u8 height;
  be u16 numRecords;
  be u16 reserved;
};

struct BTRecord {};

struct BTHeaderRec {
  be u16 treeDepth;
  be u32 rootNode;
  be u32 leafRecords;
  be u32 firstLeafNode;
  be u32 lastLeafNode;
  be u16 nodeSize;
  be u16 maxKeyLength;
  be u32 totalNodes;
  be u32 freeNodes;
  be u16 reserved1;
  be u32 clumpSize;
  u8 btreeType;
  u8 keyCompareType;
  be u32 attributes;
  be u32 reserved3[16];
};

struct BTreeNode8k {
  BTNodeDescriptor descriptor;

  u8 payload[8192 - 14 - 2 * descriptor.numRecords - 2];
  be u16 freeSpaceOffset;
  be u16 recordOffsets[descriptor.numRecords];
};

struct BTree {
  // Inline the Header node
  BTNodeDescriptor headerNodeDescriptor;
  BTHeaderRec headerNodeRecord;
  u8 headerNodeUserData[128]; // Opaque, reserved
  u8 allocationMap[headerNodeRecord.nodeSize - 256];
  be u16 freeSpaceOffset;
  be u16 recordOffsets[headerNodeDescriptor.numRecords];

  BTreeNode8k nodes[headerNodeRecord.totalNodes / 100];
};

struct HFSVolume {
    u8 preamble[1024];
    HFSPlusVolumeHeader header;
    u8 allocationFile[header.allocationFile.extents[0].blockCount * header.blockSize] @ header.allocationFile.extents[0].startBlock * header.blockSize;
    // BTree extentFile @ header.extentsFile.extents[0].startBlock * header.blockSize;
    BTree catalogFile @ header.catalogFile.extents[0].startBlock * header.blockSize;
    // BTree attributesFile @ header.attributesFile.extents[0].startBlock * header.blockSize;
    JournalInfoBlock journal_info @ header.journalInfoBlock * header.blockSize;
};

HFSVolume volume @0x00;
