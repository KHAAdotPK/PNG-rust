/*
    lib/rust/png/src/lib.rs
    Q@khaa.pk
 */

mod constants;

use std::collections::LinkedList; 
use libc::{c_uchar, c_ulong};

#[link(name = "png", kind = "dylib")]
/* Native function call */
extern {
 
     fn big_endian_read_u32(ptr: *const u8) -> u32;     
     fn in_flate(data: *const u8, data_size: usize) -> *mut InflatedData;
}

/// A C-compatible structure representing deflated (compressed) data.
///
/// This struct matches the memory layout of its C counterpart and is used for FFI operations.
/// It contains a pointer to compressed data and its size in bytes.
///
/// # Safety
/// - The `data` pointer must either be null or point to valid memory allocated by the same allocator
///   that Rust uses (typically the system allocator).
/// - The `size` must accurately reflect the allocated memory size.
#[repr(C)]
pub struct DeflatedData {
    size: c_ulong,
    data: *mut c_uchar,
}

/// Type alias for inflated (decompressed) data.
///
/// Uses the same memory representation as `DeflatedData` but semantically represents decompressed content.
pub type InflatedData = DeflatedData;

// Add Drop trait implementation for automatic cleanup

impl DeflatedData {

    /// Creates a new `DeflatedData` instance.
    ///
    /// # Arguments
    /// * `size` - The length of the data buffer in bytes
    /// * `data` - A pointer to the raw data (may be null)
    ///
    /// # Safety
    /// - The caller must ensure the pointer is valid for the given size
    /// - If not null, the pointer must be properly aligned and point to initialized memory
    pub fn new (size: c_ulong, data: *mut c_uchar) -> Self {

        Self {

            size,
            data,
        }
    }
    
    /// Safely deallocates the contained data buffer.
    ///
    /// # Safety
    /// - Must only be called when the struct owns the data pointer
    /// - After calling, the struct becomes invalid and should not be used
    unsafe fn free(&mut self) {
        if !self.data.is_null() {
            // Convert back to a Vec before dropping to properly deallocate
            let _ = Vec::from_raw_parts(self.data, self.size as usize, self.size as usize);
            // Mark as freed to prevent double-free
            self.data = std::ptr::null_mut();
            self.size = 0;
        }
    }

    /// Returns the size of the contained data in bytes.
    ///
    /// Note: Returns 0 if the data has been freed or was never allocated.
    pub fn len(&self) -> c_ulong {
        self.size        
    }
}

/// Automatic cleanup implementation for `DeflatedData`.
///
/// Ensures proper memory deallocation when the struct goes out of scope.
///
/// # Safety
/// - The drop handler is automatically called by Rust's ownership system
/// - After drop completes, the struct becomes invalid
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

/// Represents a PNG chunk containing metadata or image data.
///
/// Each PNG chunk consists of:
/// - 4-byte length (big-endian)
/// - 4-byte type name (ASCII)
/// - Variable-length data payload
/// - 4-byte CRC checksum
///
/// # Safety
/// - Uses `unsafe` operations for byte manipulation and inflation
/// - Assumes proper PNG chunk structure in input data
#[derive(Clone)]
pub struct Chunk {
     
    pub length: Vec<u8>,
    pub type_name: Vec<u8>,
    pub data: Vec<u8>,
    pub crc: Vec<u8>,
}

impl Chunk {

    /// Creates a new `Chunk` from raw byte data.
    ///
    /// # Arguments
    /// * `data` - Byte slice containing a complete PNG chunk
    ///
    /// # Panics
    /// - If input data doesn't contain a valid chunk structure
    ///
    /// # Safety
    /// - Uses unsafe pointer arithmetic to parse chunk structure
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
    
    /// Returns the chunk's data length as a big-endian u32.
    pub fn get_length (&self) -> u32 {

        unsafe { big_endian_read_u32 (self.length.as_ptr()) }
    }
  
    /// Returns the chunk type as an ASCII string.
    ///
    /// # Panics
    /// - If type name contains invalid UTF-8 sequences
    pub fn get_type_name (&self) -> String {

        std::str::from_utf8(&self.type_name).unwrap().to_string() //unsafe { big_endian_read_u32 (self.type_name.as_ptr()) }
    }
    
    /// Decompresses the chunk's data using zlib inflation.
    ///
    /// # Returns
    /// Raw pointer to decompressed data (must be properly freed by caller)
    ///
    /// # Safety
    /// - Returns a raw pointer requiring manual memory management
    /// - Assumes valid zlib-compressed data
    pub fn get_inflated_data (&self) -> *mut InflatedData {
        
        unsafe { in_flate(self.data.as_ptr(), self.get_length() as usize ) }
    }
    
    //////////////////////////////////////////////////////
    // Block containing IHDR related methods begin here //   
    //////////////////////////////////////////////////////

    /// Returns image width from IHDR chunk (big-endian).
    pub fn get_width (&self) -> u32 {

        unsafe { big_endian_read_u32 (self.data.as_ptr()) }
    }

    /// Returns image height from IHDR chunk (big-endian).
    pub fn get_height (&self) -> u32 {
       
        unsafe { big_endian_read_u32 (self.data.as_ptr().wrapping_add(4)) }
    }

    /// Returns bit depth from IHDR chunk.
    pub fn get_bit_depth (&self) -> u8 {

        self.data[8]
    }

    /// Returns color type from IHDR chunk.
    pub fn get_color_type (&self) -> u8 {

        self.data[9]
    }

    /// Returns compression method from IHDR chunk.
    pub fn get_compression_method (&self) -> u8 {

        self.data[10]
    }

    /// Returns filter method from IHDR chunk.
    pub fn get_filter_method (&self) -> u8 {

        self.data[11]
    }

    /// Returns interlace method from IHDR chunk.
    pub fn get_interlace_method (&self) -> u8 {

        self.data[12]
    }
    ////////////////////////////////////////////////////
    // Block containing IHDR related methods end here //   
    ////////////////////////////////////////////////////
}

/// Represents a complete PNG image file.
///
/// Contains:
/// - 8-byte PNG signature
/// - Linked list of chunks (IHDR, PLTE, IDAT, IEND, etc.)
#[derive(Clone)]
pub struct Png {
    
    pub signature: Vec<u8>,            
    pub chunks: LinkedList<Chunk>,
}

impl Png {

    /// Creates a new `Png` instance by parsing raw PNG file data.
    ///
    /// # Arguments
    /// * `data` - A byte vector containing the complete PNG file data
    ///
    /// # Returns
    /// A new `Png` struct containing:
    /// - The 8-byte PNG signature
    /// - A linked list of parsed chunks (IHDR, IDAT, IEND, etc.)
    ///
    /// # Panics
    /// - Will not panic but returns empty structure if input is shorter than PNG signature
    ///
    /// # Safety
    /// - Uses unsafe operations for big-endian number parsing
    /// - Assumes valid PNG file structure in input data
    ///
    /// # Example
    /// ```rust
    /// let png_data = std::fs::read("image.png").unwrap();
    /// let png = Png::new(png_data);
    /// ```
    ///
    /// # Implementation Notes
    /// 1. Validates minimum data length (8 bytes for signature)
    /// 2. Extracts PNG signature (first 8 bytes)
    /// 3. Iteratively parses chunks:
    ///    - Each chunk has: length (4B), type (4B), data (variable), CRC (4B)
    ///    - Uses big-endian format for length fields
    /// 4. Builds linked list of chunks
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

    /// Returns the PNG signature (first 8 bytes).
    pub fn get_signature (&self) -> Vec<u8> {

        self.signature.clone()
    }

    /// Returns a reference to the list of chunks.
    pub fn get_chunks (&self) -> &LinkedList<Chunk> {

        &self.chunks
    }
       
    /// Collects and concatenates all `IDAT` chunk data from the PNG file.
    ///
    /// This method iterates through all chunks in the PNG and extracts the raw byte data
    /// from every `IDAT` chunk, combining them into a single contiguous `Vec<u8>`.
    ///
    /// # Notes
    /// - The `IDAT` chunks contain the compressed image pixel data in PNG format.
    /// - This method does **not** decompress the data; it simply concatenates the raw `IDAT` payloads.
    /// - Asserts that each chunk's declared length matches its actual data length.
    ///
    /// # Returns
    /// A `Vec<u8>` containing the concatenated raw data from all `IDAT` chunks.
    ///
    /// # Panics
    /// - If any `IDAT` chunk's declared length (from its header) does not match its actual data length.
    ///
    /// # Example
    /// ```rust
    /// let png = Png::from_file("image.png").unwrap();
    /// let idat_data = png.get_all_idat_data();
    /// // Further processing (e.g., decompression with a zlib decoder)
    /// ```
    pub fn get_all_idat_data(&self) -> Vec<u8> {

        let mut all_idat_data = Vec::new();

        let mut iter = self.chunks.iter();

        while let Some(chunk) = iter.next() {
                 
            if chunk.get_type_name() == "IDAT" {
                
                // Check if it matches the actual data length
                assert_eq!(chunk.get_length() as usize, chunk.data.len());

                all_idat_data.extend_from_slice(&chunk.data);                             
            }
        }

        all_idat_data
    }

    /// Decompresses raw PNG `IDAT` chunk data using zlib inflation.
    ///
    /// This method takes a byte slice containing compressed image data (typically from concatenated `IDAT` chunks)
    /// and returns a raw pointer to the decompressed data. The caller is responsible for managing the memory
    /// of the returned `InflatedData`.
    ///
    /// # Safety
    /// - **Unsafe Operation**: This function calls an `unsafe` FFI function (`in_flate`) and returns a raw pointer.
    /// - **Memory Management**: The caller must ensure the returned `*mut InflatedData` is properly freed to avoid leaks.
    /// - **Input Validity**: The input `data` must be valid zlib-compressed data, or behavior is undefined.
    ///
    /// # Parameters
    /// - `data`: A byte slice containing the compressed PNG image data (usually from `IDAT` chunks).
    ///
    /// # Returns
    /// A raw pointer (`*mut InflatedData`) to the decompressed data, or a null pointer on failure.
    ///
    /// # Example
    /// ```rust
    /// # use std::ptr;
    /// # struct InflatedData;
    /// # unsafe fn in_flate(_: *const u8, _: usize) -> *mut InflatedData { ptr::null_mut() }
    /// # impl YourStruct {
    /// # pub fn get_inflated_data(&self, data: &[u8]) -> *mut InflatedData {
    /// #     unsafe { in_flate(data.as_ptr(), data.len()) }
    /// # }
    /// # }
    /// let png = YourStruct::new();
    /// let idat_data = png.get_all_idat_data(); // Get compressed data
    /// let inflated = png.get_inflated_data(&idat_data);
    ///
    /// // SAFETY: Check for null and ensure proper cleanup later.
    /// assert!(!inflated.is_null());
    /// # // Hypothetical cleanup (implementation-specific):
    /// # unsafe { libc::free(inflated as *mut libc::c_void); }
    /// ```
    ///
    /// # Implementation Notes
    /// - The underlying `in_flate` function is expected to:
    ///   - Use zlib-compatible decompression.
    ///   - Return null on failure (e.g., corrupt input or allocation error).
    /// - For safer alternatives, consider wrapping the result in a `Box` or using Rust's `flate2` crate.    
    pub fn get_inflated_data (&self, data: &[u8]) -> *mut InflatedData {

        unsafe { in_flate(data.as_ptr(), data.len()) }
    }

    /// Traverses and prints metadata for all chunks in the PNG file.
    ///
    /// This method iterates through each chunk in the PNG and logs its type, length, and
    /// (for `IHDR` chunks) critical image properties like width, height, bit depth, etc.
    /// It is primarily used for debugging and inspecting PNG structure.
    ///
    /// # Notes
    /// - For `IHDR` chunks, detailed information is printed, including:
    ///   - Image dimensions (width/height)
    ///   - Bit depth (1, 2, 4, 8, or 16)
    ///   - Color type (0=Grayscale, 2=Truecolor, 3=Indexed, 4=Grayscale+Alpha, 6=Truecolor+Alpha)
    ///   - Compression/filter/interlace methods (typically 0 for standard PNG)
    /// - Other chunk types (e.g., `IDAT`, `PLTE`) are logged only by type and length.
    ///
    /// # Returns
    /// Always returns `true` (presumably for legacy/chaining purposes; consider refactoring to `()` if unused).
    ///
    /// # Example
    /// ```rust
    /// let png = Png::from_file("image.png").unwrap();
    /// png.traverse(); // Prints chunk metadata to stdout
    /// ```
    ///
    /// # Implementation Details
    /// - **IDAT Handling**: The method does not process `IDAT` data but notes its storage format:
    ///   - For `Color type = 2` (Truecolor), pixels are contiguous RGB triples (3 bytes/pixel for 8-bit depth).
    ///   - For `Color type = 3` (Indexed), pixels are palette indices (1, 2, 4, or 8 bits/index).
    /// - **Assertions**: No runtime checks are performed on chunk validity beyond logging.    
    pub fn traverse(&self) -> bool {

        let mut iter = self.chunks.iter();

        while let Some(chunk) = iter.next() {
           
            //println!("Length = {}", chunk.get_length() );
            println!("Type = [ {} {} {} {} ],{} and Chunk Length = {}", 
                chunk.type_name[0], 
                chunk.type_name[1], 
                chunk.type_name[2], 
                chunk.type_name[3],                             
                chunk.get_type_name(),                            
                chunk.get_length()                            
            );

            if chunk.get_type_name() == "IHDR" {

                println!("IHDR data --> Width = {}, Height = {}, Bit Depth = {}, Color Type = {}, Compression Method = {}, Filter Method = {}, Interlace Method = {}", chunk.get_width(), chunk.get_height(), chunk.get_bit_depth(), chunk.get_color_type(), chunk.get_compression_method(), chunk.get_filter_method(), chunk.get_interlace_method());                
                
                /* 
                    //8th octet, Bit depth: The number of bits per sample or per palette index (not per pixel). Valid values are 1, 2, 4, 8, and 16. Not all values are allowed for all colour types.
                    //9th octet, Color type: The number of channels per pixel. Valid values are 0, 2, 3, 4, and 6. Not all values are allowed for all colour types.
                    //10th octet, Compression method: The compression method used. Valid values are 0 and 1. Not all values are allowed for all colour types.
                    //11th octet, Filter method: The filter method used. Valid values are 0 and 1. Not all values are allowed for all colour types.
                    //12th octet, Interlace method: The interlace method used. Valid values are 0 and 1. Not all values are allowed for all colour types.
                 */                
                /*
                    Implementation details: How to handle the IDAT chunk? based on the Bit depth and Color type.
                    --------------------------------------------------------------------------------------------
                
                    Which Bit depth and Color type are important for us?
                    Bit depth = 8, 16, Color type = 2 // IDAT is contiguois array of a 3 bytes per pixel. Each pixel is an R,G,B triple                                                                    
                    Bit depth = 1, 2, 4, 8, Color type = 3 // IDAT is contiguois array of 1, 2, 4, or b bit entities. Each entity is a palette index; a PLTE chunk shall appear.
                 */ 
            }
        }

        true
    }
}
