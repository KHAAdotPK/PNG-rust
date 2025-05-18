/*
    lib/rust/png/src/lib.rs
    Q@khaa.pk
 */

mod constants;

use std::collections::LinkedList; 
use libc::{c_uchar, c_ulong};

// Define the Rust equivalent of the C struct
#[repr(C)]
pub struct DeflatedData {
    size: c_ulong,
    data: *mut c_uchar,
}

// Add Drop trait implementation for automatic cleanup

impl DeflatedData {

    pub fn new (size: c_ulong, data: *mut c_uchar) -> Self {

        Self {

            size,
            data,
        }
    }

    // Free the allocated memory
    unsafe fn free(&mut self) {
        if !self.data.is_null() {
            // Convert back to a Vec before dropping to properly deallocate
            let _ = Vec::from_raw_parts(self.data, self.size as usize, self.size as usize);
            // Mark as freed to prevent double-free
            self.data = std::ptr::null_mut();
            self.size = 0;
        }
    }

    pub fn len(&self) -> c_ulong {
        self.size        
    }
}

// Separate Drop implementation
impl Drop for DeflatedData {
    fn drop(&mut self) {
        unsafe {
            self.free();

            self.size = 0;
            self.data = std::ptr::null_mut();            
        }

        /*
            Uncomment the following line to print a message when DeflatedData is dropped
        */
        //println!("DeflatedData dropped");
    }
}

pub type InflatedData = DeflatedData;

#[link(name = "png", kind = "dylib")]
/* Native function call */
extern {
 
     fn big_endian_read_u32(ptr: *const u8) -> u32;     
     fn in_flate(data: *const u8, data_size: usize) -> *mut InflatedData;
}

#[derive(Clone)]
pub struct Chunk {
     
    pub length: Vec<u8>,
    pub type_name: Vec<u8>,
    pub data: Vec<u8>,
    pub crc: Vec<u8>,
}

impl Chunk {

    pub fn new (data: Vec<u8>) -> Self {

        if data.len() > 0 {
                        
            Self {

                length: data[0 .. 4].to_vec(),
                type_name: data[4 .. 8].to_vec(),
                data: data[8 .. unsafe { big_endian_read_u32 (data[0 .. 4].as_ptr()) } as usize + 8 ].to_vec(),
                crc: data[ unsafe { big_endian_read_u32 (data[0 .. 4].as_ptr()) } as usize + 8 .. unsafe { big_endian_read_u32 (data[0 .. 4].as_ptr()) } as usize + 4 + 8 ].to_vec(),                
            }
        }
        else {
            
            Self {
                
                length: Vec::new(),
                type_name: Vec::new(),
                data: Vec::new(),
                crc: Vec::new(),                
            }
        }
    }
    
    // This method will call big_endian_read_u32 and return the length of the chunk
    pub fn get_length (&self) -> u32 {

        unsafe { big_endian_read_u32 (self.length.as_ptr()) }
    }

    // This method will convert the type_name vector to a string and return it
    pub fn get_type_name (&self) -> String {

        std::str::from_utf8(&self.type_name).unwrap().to_string() //unsafe { big_endian_read_u32 (self.type_name.as_ptr()) }
    }

    // This method will take Chunk::data and inflate and return it
    pub fn get_inflated_data (&self) -> *mut InflatedData {
        
        unsafe { in_flate(self.data.as_ptr(), self.get_length() as usize ) }
    }
    
    //////////////////////////////////////////////////////
    // Block containing IHDR related methods begin here //   
    //////////////////////////////////////////////////////
    pub fn get_width (&self) -> u32 {

        unsafe { big_endian_read_u32 (self.data.as_ptr()) }
    }

    pub fn get_height (&self) -> u32 {
       
        unsafe { big_endian_read_u32 (self.data.as_ptr().wrapping_add(4)) }
    }

    pub fn get_bit_depth (&self) -> u8 {

        self.data[8]
    }

    pub fn get_color_type (&self) -> u8 {

        self.data[9]
    }

    pub fn get_compression_method (&self) -> u8 {

        self.data[10]
    }

    pub fn get_filter_method (&self) -> u8 {

        self.data[11]
    }

    pub fn get_interlace_method (&self) -> u8 {

        self.data[12]
    }
    ////////////////////////////////////////////////////
    // Block containing IHDR related methods end here //   
    ////////////////////////////////////////////////////
}

#[derive(Clone)]
pub struct Png {
    
    pub signature: Vec<u8>,            
    pub chunks: LinkedList<Chunk>,
}

impl Png {

    pub fn new (data: Vec<u8>) -> Self {

        let mut head: LinkedList<Chunk> = LinkedList::new();
                        
        //let mut signature = vec![0; constants::LENGTH_OF_SIGNATURE];

        // Check if there are enough bytes in the data vector.
        if data.len() >= constants::LENGTH_OF_SIGNATURE {
            let mut signature = vec![0; constants::LENGTH_OF_SIGNATURE];
            // Copy the first 8 bytes from data to signature.
            signature.clone_from_slice(&data[0..constants::LENGTH_OF_SIGNATURE]);

            let mut index: usize = constants::LENGTH_OF_SIGNATURE;
        
            while index < data.len() {
            
                let chunk = Chunk::new(

                    (&data[index .. (index + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(data[index .. (index + 4)].as_ptr()) } as usize + constants::LENGTH_OF_CRC_FIELD)]).to_vec()
                );

                head.push_back(chunk);
            
                index = index + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(data[index .. (index + 4)].as_ptr()) } as usize + constants::LENGTH_OF_CRC_FIELD;
            }

            Self {

                signature,                                    
                chunks: head,
            }

        } else {

            // If data is not long enough, handle the error or set a default value.
            // For example, you might panic, return an error, or set a default signature.
            /*panic!("Not enough bytes in data vector to form a valid PNG signature");*/

            Self {

                signature: Vec::new(),
                chunks: head,
            }
        }   
        
        //let mut n: usize = 0;

        /*let mut index: usize = constants::LENGTH_OF_SIGNATURE;        
        while index < data.len() {
            
            let chunk = Chunk::new(

                (&data[index .. (index + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(data[index .. (index + 4)].as_ptr()) } as usize + constants::LENGTH_OF_CRC_FIELD)]).to_vec()
            );

            head.push_back(chunk);
            
            index = index + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(data[index .. (index + 4)].as_ptr()) } as usize + constants::LENGTH_OF_CRC_FIELD;
        } */
       
        /*Self {

            signature,                                    
            chunks: head,
        }*/
    }

    pub fn get_inflated_data (&self, data: &[u8]) -> *mut InflatedData {

        unsafe { in_flate(data.as_ptr(), data.len()) }
    }
}
