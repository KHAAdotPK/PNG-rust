/*
    lib/rust/png/src/constants.rs
    Q@khaa.pk
 */
#![allow(dead_code)]

pub const LENGTH_OF_SIGNATURE: usize = 8;
//pub const LENGTH_OF_IHDR_DATA: usize = /*13;*/ [u8; LENGTH_OF_] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
pub const LENGTH_OF_LENGTH_FIELD: usize = 4;
pub const LENGTH_OF_TYPE_FIELD: usize = 4;
pub const LENGTH_OF_CRC_FIELD: usize = 4;
pub const LENGTH_OF_THREE_FIELDS: usize = LENGTH_OF_LENGTH_FIELD + LENGTH_OF_TYPE_FIELD + LENGTH_OF_CRC_FIELD;
pub const PNG_SIGNATURE: [u8; LENGTH_OF_SIGNATURE] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
pub const PNG_IHDR_TYPE_SIGNATURE: [u8; LENGTH_OF_TYPE_FIELD] = [0x49, 0x48, 0x44, 0x52];
pub const LENGTH_OF_IHDR_DATA: [u8; LENGTH_OF_LENGTH_FIELD] = [0x00, 0x00, 0x00, 0x0D];
pub const IHDR_DATA_FOR_UNCOMPRESSED_FILE : [u8; 5] = [0x08, 0x02, 0x00, 0x00, 0x00]; 
pub const LENGTH_OF_IEND_DATA: [u8; LENGTH_OF_LENGTH_FIELD] = [0x00, 0x00, 0x00, 0x00];
pub const PNG_IEND_TYPE_SIGNATURE: [u8; LENGTH_OF_TYPE_FIELD] = [0x49, 0x45, 0x4E, 0x44];
pub const PNG_IDAT_TYPE_SIGNATURE: [u8; LENGTH_OF_TYPE_FIELD] = [0x49, 0x44, 0x41, 0x54];