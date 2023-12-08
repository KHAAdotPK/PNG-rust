/*
    lib/rust/png/src/constants.rs
    Q@khaa.pk
 */
#![allow(dead_code)]

pub const LENGTH_OF_SIGNATURE: usize = 8;
pub const LENGTH_OF_LENGTH_FIELD: usize = 4;
pub const LENGTH_OF_TYPE_FIELD: usize = 4;
pub const LENGTH_OF_CRC_FIELD: usize = 4;
pub const LENGTH_OF_THREE_FIELDS: usize = LENGTH_OF_LENGTH_FIELD + LENGTH_OF_TYPE_FIELD + LENGTH_OF_CRC_FIELD;