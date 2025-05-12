#![allow(non_upper_case_globals, non_snake_case)]
#![deny(dead_code)]

pub const kHFSPlusSigWord: u16 = u16::from_be_bytes(*b"H+");
pub const kHFSXSigWord: u16 = u16::from_be_bytes(*b"HX");

#[repr(C)]
pub struct HFSPlusVolumeHeader {
    pub signature: u16,
    pub version: u16,
    pub attributes: u32,
    pub lastMountedVersion: u32,
    pub journalInfoBlock: u32,

    pub createDate: u32,
    pub modifyDate: u32,
}
