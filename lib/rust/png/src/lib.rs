/*
    lib/rust/png/src/lib.rs
    Q@khaa.pk
 */

// Declare modules at the crate root
pub mod constants;
pub mod png_core;

// Re-export everything from png_core
pub use png_core::*;

use std::{fs::File, io::Write, collections::LinkedList, path::Path, ptr}; 
use libc::{c_uchar, c_ulong, c_uint};

#[link(name = "png", kind = "dylib")]
/* Native function call */
extern {
 
     fn big_endian_read_u32(ptr: *const u8) -> u32;     
     fn in_flate(data: *const u8, data_size: usize) -> *mut InflatedData;
     fn de_flate(data: *const u8, data_size: usize) -> *mut DeflatedData;
     fn update_crc(crc: c_ulong, buf: *const c_uchar, len: c_uint) -> c_ulong;
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
#[derive(Clone, Debug)]
pub struct DeflatedData {
    // Fields
    pub size: c_ulong,
    pub data: *mut c_uchar,
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
// Add these derive attributes to your Chunk struct
#[derive(Debug, PartialEq, Clone)]
pub struct Chunk {
    // Fields 
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

    pub fn get_crc (&self) -> u32 {

        unsafe { big_endian_read_u32 (self.crc.as_ptr()) }
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

    pub fn get_data (&self) -> Vec<u8> {

        self.data.clone()
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
    // Fields
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

                //intln! ("Chunk gone amd index = {}", index);
            
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

    pub fn get_chunk_by_type (&self, type_name: &str) -> Option<&Chunk> {

        self.chunks.iter().find(|chunk| chunk.get_type_name() == type_name)
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
    pub fn get_all_idat_data_as_vec(&self) -> Vec<u8> {

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

    /// Retrieves all IDAT chunk data from the PNG file as a single deflated data block.
    ///
    /// This function:
    /// 1. Iterates through all chunks in the PNG file
    /// 2. Collects the raw data from all IDAT chunks into a contiguous buffer
    /// 3. Packages the data in a `DeflatedData` structure containing:
    ///    - The total size of all IDAT data combined (as u32)
    ///    - A raw pointer to the deflated data bytes
    ///
    /// # Returns
    /// A raw pointer to a heap-allocated `DeflatedData` structure containing:
    /// - `size`: Total size of all IDAT data in bytes
    /// - `data`: Pointer to the raw deflated data bytes (all IDAT chunks concatenated)
    ///
    /// # Safety
    /// The caller is responsible for:
    /// - Properly freeing the memory allocated for the `DeflatedData` structure
    /// - Ensuring the data pointer is not used after the structure is freed
    /// - The data pointer becomes invalid if the original PNG structure is modified or dropped
    ///
    /// # Panics
    /// - If any IDAT chunk's declared length doesn't match its actual data length
    ///
    /// # Notes
    /// - The returned data is exactly as it appears in the IDAT chunks (still deflated/compressed)
    /// - IDAT chunks are concatenated in the order they appear in the file
    /// - Non-IDAT chunks are ignored
    pub fn get_all_idat_data_as_DeflatedData(&self) -> *mut DeflatedData {
        let mut all_idat_data = Vec::new();
    
        let mut iter = self.chunks.iter();
    
        while let Some(chunk) = iter.next() {
            if chunk.get_type_name() == "IDAT" {
                // Check if it matches the actual data length
                assert_eq!(chunk.get_length() as usize, chunk.data.len());
                all_idat_data.extend_from_slice(&chunk.data);                             
            }
        }
    
        let deflated_data = DeflatedData {
            size: all_idat_data.len() as u32,  // Convert usize to u32
            data: all_idat_data.as_mut_ptr(),  // Get raw pointer from Vec
        };
    
        // Allocate on heap and return pointer
        Box::into_raw(Box::new(deflated_data))
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
    
    /// (Raw Pointer Version)
    /// Compresses (deflates) the given inflated (raw) image data using zlib compression.
    ///
    /// This function takes a raw pointer to an `InflatedData` structure and returns a pointer
    /// to a newly allocated compressed (deflated) data buffer in DEFLATE/zlib format.
    ///
    /// # Arguments
    /// * `data` - A raw pointer to an `InflatedData` structure containing:
    ///   - `size`: The size of the uncompressed data in bytes
    ///   - `data`: Pointer to the raw uncompressed pixel data
    ///
    /// # Returns
    /// A raw pointer to a heap-allocated `DeflatedData` structure containing:
    /// - `size`: The size of the compressed data in bytes
    /// - `data`: Pointer to the compressed data buffer
    ///
    /// # Safety
    /// This function is marked `unsafe` because:
    /// - It dereferences a raw pointer (`data`)
    /// - The caller must ensure:
    ///   - The input pointer is valid and properly aligned
    ///   - The `InflatedData` structure hasn't been freed
    ///   - The data pointer within `InflatedData` points to valid memory of the specified size
    /// - The caller becomes responsible for:
    ///   - Managing the lifetime of the input data (this function does NOT free it)
    ///   - Freeing the returned `DeflatedData` structure
    ///
    /// # Memory Management
    /// - Does NOT take ownership of the input pointer
    /// - The caller must ensure proper cleanup of both input and output data
    ///
    /// # Panics
    /// - May panic if the compression fails (though current implementation assumes success)
    ///
    /// # Notes
    /// - Uses zlib/DEFLATE compression (standard for PNG)
    /// - Prefer `get_deflated_data_from_boxed_inflated_data` for safer memory handling
    pub fn get_deflated_data_from_inflated_data (&self, data: *mut InflatedData) -> *mut DeflatedData {
        
        unsafe {
             
            de_flate((*data).data as *const u8, (*data).size as usize) 
        }
    }

    /// (Owned Version)
    /// Compresses (deflates) the given owned inflated image data using zlib compression.
    ///
    /// This function takes ownership of a `Box<InflatedData>` and returns a pointer
    /// to a newly allocated compressed (deflated) data buffer in DEFLATE/zlib format.
    ///
    /// # Arguments
    /// * `data` - A boxed `InflatedData` structure containing:
    ///   - `size`: The size of the uncompressed data in bytes
    ///   - `data`: Pointer to the raw uncompressed pixel data
    ///
    /// # Returns
    /// A raw pointer to a heap-allocated `DeflatedData` structure containing:
    /// - `size`: The size of the compressed data in bytes
    /// - `data`: Pointer to the compressed data buffer
    ///
    /// # Safety
    /// This function is marked `unsafe` because:
    /// - It internally dereferences raw pointers
    /// - The caller must ensure:
    ///   - The box contains valid, initialized data
    /// - The caller becomes responsible for:
    ///   - Freeing the returned `DeflatedData` structure
    ///
    /// # Memory Management
    /// - Takes ownership of the input data (will be freed when function exits)
    /// - The returned pointer must still be manually managed
    ///
    /// # Panics
    /// - May panic if the compression fails (though current implementation assumes success)
    ///
    /// # Notes
    /// - Uses zlib/DEFLATE compression (standard for PNG)
    /// - Safer than raw pointer version as it guarantees input data validity
    /// - Still returns raw pointer for FFI compatibility
    pub fn get_deflated_data_from_boxed_inflated_data (&self, data: Box<InflatedData>) -> *mut DeflatedData {
        
        unsafe {
             
            de_flate((*data).data as *const u8, (*data).size as usize) 
        }
    }

    /**
     * Removes PNG filter bytes from inflated IDAT data and extracts raw pixel data.
     * 
     * This method processes PNG image data that has been decompressed from IDAT chunks,
     * removing the filter type bytes that appear at the beginning of each scanline and
     * extracting the actual pixel RGB values.
     * 
     * # PNG Filtering Context
     * PNG images use a filtering system where each scanline (row) begins with a filter
     * type byte (0-4) that indicates how the pixel data in that row has been filtered.
     * Filter type 0 means "None" - no filtering applied, so pixel data is raw.
     * 
     * # Parameters
     * * `inflated_data` - Raw pointer to InflatedData containing decompressed IDAT data
     * 
     * # Returns
     * * `*mut InflatedData` - Boxed pointer to new InflatedData containing filtered pixel data,
     *   or null data on error
     * 
     * # Supported Color Types
     * * Color type 2: Truecolor RGB (3 bytes per pixel)
     * * Color type 3: Indexed color (1 byte per pixel)
     * 
     * # Safety
     * This function uses unsafe code to dereference raw pointers and create slices from
     * raw memory. The caller must ensure the inflated_data pointer is valid and points
     * to properly initialized InflatedData.
     * 
     * # Error Handling
     * Returns InflatedData with null pointer and size 0 if:
     * - IHDR chunk is missing
     * - Unsupported color type
     * - Row data extends beyond available data bounds
     */
    pub fn remove_filter_bytes_from_inflated_data(&self, inflated_data: *mut InflatedData) -> *mut InflatedData {

        let ihdr_chunk = self.get_chunk_by_type("IHDR");

        if ihdr_chunk.is_none() {

            //panic!("IHDR chunk not found in PNG file");

            let new_inflated_data: InflatedData = InflatedData {
                size: 0 as c_ulong,  // Convert usize to u32
                data: ptr::null_mut(),  // Get raw pointer from Vec
            };

            return Box::into_raw(Box::new(new_inflated_data));
        }

        /*
            Filtering Method IHDR field
            ---------------------------
            In PNG format, the filter method in the IHDR chunk is typically just one byte that indicates the filtering algorithm used.
            For standard PNG files, this value is almost always 0, which indicates the default PNG filtering method.
            With filter method 0, each scanline begins with a filter type byte which has value None(0) which means no filtering method is applied.
            So 0 is standard PNG filtering (this is the normal case for virtually all PNG files).
            Values 1-255 are reserved for future use, but in practice, you'll almost never see anything other than 0

            The actual filter type bytes (0-4) that appear at the beginning of each scanline are determined during the encoding process and can vary from line to line within the same image, 
            but the filter method in the IHDR is just indicating which filtering system is being used overall.
         */

        let chunk= ihdr_chunk.unwrap();

        let width = chunk.get_width() as usize;
        let height = chunk.get_height() as usize;

        let bytes_per_pixel = match chunk.get_color_type() {
            2 => 3, // Truecolor: RGB
            3 => 1, // Indexed: Palette index
            _ => {        
                let new_inflated_data: InflatedData = InflatedData {
                    size: 0 as c_ulong,  // Convert usize to u32
                    data: ptr::null_mut(),  // Get raw pointer from Vec
                };

                return Box::into_raw(Box::new(new_inflated_data));
            } 
        };

        let row_stride: usize;

        let data_ptr: *mut u8;
        let data_size: usize;

        let data_slice: &[u8];

        /*
            place 1/2
            Declare return value her
         */
        let filtered_data_size = width * height * bytes_per_pixel;
        let mut filtered_data_vec: Vec<u8> = Vec::with_capacity(filtered_data_size);
        
        unsafe {            
            data_ptr = (*inflated_data).data;
            data_size = (*inflated_data).size as usize;
            data_slice = std::slice::from_raw_parts(data_ptr, data_size);
            row_stride = (width * bytes_per_pixel) + 1;

            for row in 0..height {
                let row_start = row * row_stride;
                let row_end = row_start + row_stride;
            
                if row_end <= data_slice.len() {
                    let row_data = &data_slice[row_start..row_end];
                
                    // First byte is filter type
                    let filter_type = row_data[0];

                    if filter_type == 0 {        
                    }
                
                    // Process pixel data (skip first byte which is filter type)
                    for (pixel_idx, pixel_chunk) in row_data[1..].chunks_exact(bytes_per_pixel).enumerate() {
                        let r = pixel_chunk[0];
                        let g = pixel_chunk[1]; 
                        let b = pixel_chunk[2];

                        filtered_data_vec.push(r);
                        filtered_data_vec.push(g);
                        filtered_data_vec.push(b);

                        // Process RGB values as needed
                        //println!("Row {}, Pixel {}: RGB({}, {}, {})", row + 1, pixel_idx + 1, r, g, b);

                        /*  
                            place 2/2
                            initializze the returned value here...                            
                         */
                    }
                } else {
                    eprintln!("Png::remove_filter_bytes_from_inflated_data() Error: row {} extends beyond available data", row);

                    let new_inflated_data: InflatedData = InflatedData {
                        size: 0 as c_ulong,  // Convert usize to u32
                        data: ptr::null_mut(),  // Get raw pointer from Vec
                    };

                    return Box::into_raw(Box::new(new_inflated_data));                    
                }
            }
        }

        // Convert Vec to raw pointer for return
        let filtered_data_ptr = filtered_data_vec.as_mut_ptr();
        let filtered_size = filtered_data_vec.len();

        // Prevent Vec from deallocating the memory when it goes out of scope
        std::mem::forget(filtered_data_vec);

        let new_inflated_data: InflatedData = InflatedData {
            size: filtered_size as c_ulong,  // Convert usize to u32
            data: filtered_data_ptr,        // Get raw pointer from Vec
        };

        return Box::into_raw(Box::new(new_inflated_data));
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

                /*println!("What this value means for compression method? {}", chunk.get_compression_method());*/
                
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

    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {

        let mut file = File::create(path)?;     

        file.write_all(&self.signature)?;
        
        let mut iter = self.chunks.iter();

        while let Some(chunk) = iter.next() {

            file.write_all(&chunk.get_length().to_be_bytes())?;
            file.write_all(&chunk.get_type_name().as_bytes())?;
            file.write_all(&chunk.get_data())?;
            file.write_all(&chunk.get_crc().to_be_bytes())?;
        }
    
        Ok(())
    }

    /**
     * Validates whether the PNG image matches the specified color type and bit depth.
     * 
     * This method examines the IHDR (Image Header) chunk of the PNG file to determine
     * if the image's color type and bit depth match the provided parameters. The IHDR
     * chunk contains critical image metadata including dimensions, color type, and bit depth.
     * 
     * # Parameters
     * * `color_type` - The expected color type value (0-6 according to PNG specification):
     *   - 0: Grayscale
     *   - 2: RGB (Truecolor)
     *   - 3: Palette
     *   - 4: Grayscale with Alpha
     *   - 6: RGB with Alpha (RGBA)
     * * `bit_depth` - The expected bit depth per sample (1, 2, 4, 8, or 16 bits)
     * 
     * # Returns
     * * `true` - If the PNG's color type and bit depth both match the specified values
     * * `false` - If either value doesn't match, or if the IHDR chunk is missing/invalid
     * 
     * # Example
     * ```rust
     * let png = Png::new(buffer);
     * 
     * // Check if PNG is 8-bit RGB (truecolor)
     * if png.match_color_type_and_bit_depth(2, 8) {
     *     println!("PNG is 8-bit RGB format");
     * } else {
     *     println!("PNG format not supported");
     * }
     * ```
     * 
     * # Note
     * This method is typically used for format validation before processing PNG data,
     * as different color types and bit depths require different decoding strategies.
     */
    pub fn match_color_type_and_bit_depth(&self, color_type: u8, bit_depth: u8) -> bool {

        let chunk: Option<&Chunk> = self.get_chunk_by_type("IHDR");

        match chunk {

            Some(chunk) => {

                chunk.get_color_type() == color_type && chunk.get_bit_depth() == bit_depth
            },

            None => false
        }

        /*if chunk.is_none() {

            return false;
        }  

        let chunk = chunk.unwrap();

        chunk.get_color_type() == color_type && chunk.get_bit_depth() == bit_depth*/
    }
}

/**
 * Creates an uncompressed PNG file from inflated image data.
 * 
 * This function constructs a complete PNG file by building all required chunks:
 * - PNG Signature (8 bytes): Standard PNG file identifier
 * - IHDR Chunk: Image header containing width, height, and color information
 * - IDAT Chunk: Image data containing the actual pixel data (uncompressed)
 * - IEND Chunk: End-of-file marker
 * 
 * Each chunk follows the PNG specification format:
 * [Length (4 bytes)] [Type (4 bytes)] [Data (variable)] [CRC-32 (4 bytes)]
 * 
 * # Parameters
 * * `width` - Image width in pixels (must be > 0)
 * * `height` - Image height in pixels (must be > 0) 
 * * `inflated_data` - Raw pointer to InflatedData structure containing:
 *   - `data`: Pointer to uncompressed pixel data
 *   - `size`: Size of the pixel data in bytes
 *   - Must not be null and data must be valid for the specified size
 * 
 * # Returns
 * * `Some(Png)` - Successfully created PNG file structure
 * * `None` - If inflated_data is null or invalid
 * 
 * # Safety
 * This function is unsafe because it:
 * - Dereferences raw pointers without null checks beyond the initial validation
 * - Assumes inflated_data structure and its data pointer are valid
 * - Uses FFI calls to native C functions (update_crc, big_endian_read_u32)
 * 
 * # PNG Structure Created
 * ```
 * PNG Signature (8 bytes)
 * IHDR Chunk:
 *   - Length: 13 bytes
 *   - Type: "IHDR"
 *   - Data: width(4) + height(4) + bit_depth(1) + color_type(1) + 
 *           compression(1) + filter(1) + interlace(1)
 *   - CRC: 4 bytes
 * IDAT Chunk:
 *   - Length: size of inflated_data
 *   - Type: "IDAT" 
 *   - Data: raw pixel data from inflated_data
 *   - CRC: 4 bytes
 * IEND Chunk:
 *   - Length: 0 bytes
 *   - Type: "IEND"
 *   - Data: none
 *   - CRC: 4 bytes
 * ```
 * 
 * # Example Usage
 * ```rust
 * let width = 254u32;
 * let height = 344u32;
 * let inflated_data = get_inflated_data(); // Your data source
 * 
 * if let Some(png) = create_uncompressed_png(width, height, inflated_data) {
 *     // PNG successfully created
 *     png.save_to_file("output.png");
 * } else {
 *     eprintln!("Failed to create PNG: invalid data");
 * }
 * ```
 * 
 * # Notes
 * - All multi-byte values are stored in big-endian format per PNG specification
 * - CRC calculations use the standard PNG CRC-32 algorithm
 * - The function assumes RGB color type (2) with 8-bit depth for uncompressed data
 * - Memory allocation is pre-calculated for optimal performance
 */
pub fn create_png_from_deflated_data(width: u32, height: u32, inflated_data: *mut InflatedData, out_put_file_path: &Path) -> Option<Png> {
    unsafe {

        if ((*inflated_data).data.is_null() || (*inflated_data).len() == 0) || (width == 0 || height == 0) {

            return None;
        }

        let mut size = 0;

        //  Setting up capacity fo all PNG chunks namely PNG-Signature + IHDR + IDAT + IEND
        ////////////////////////////////////////////////////////////////////////////////////////////////
        // 1. Add PNG signature
        size = size + constants::LENGTH_OF_SIGNATURE;

        println! ("SIZE = {}", size); // 8

        // 2. Add Chunk size for IHDR
        size = size + constants::LENGTH_OF_THREE_FIELDS;        
        size = size + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize };

        /*println! ("SIZE = {}", size); // 25 + 8 = 33*/
        
        /*println! ("-----------------------------------------------> {}",  unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize });*/


        // 3. Add Chunk size for IDAT
        size = size + constants::LENGTH_OF_THREE_FIELDS;
        size = size + (*inflated_data).size as usize;

        /*println! ("SIZE = {}, {}", size, (*inflated_data).size); // 33 + 12 +  262472 = 262517*/

        // 4. Add Chunk size for IEND
        size = size + constants::LENGTH_OF_THREE_FIELDS; // No data, length must be set to zero
        /////////////////////////////////////////////////////////////////////////////////////////////////

        /*println! ("SIZE = {}", size); // 262517 + 12 = 262529*/
        
        // Create a vector with the appropriate capacity
        let mut buffer: Vec<u8> = Vec::with_capacity(size);

        println! ( "Buffer = {}", buffer.len() );
        println! ( "Capacity = {}", buffer.capacity() );

        // Start of PNG Signature
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // 1. Add PNG signature        
        buffer.extend_from_slice(&constants::PNG_SIGNATURE);
        // Lets use the debug format specifier {:?}, to debug and print slice containing the PNG signature bytes.
        /*println!("PNG Signature ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE]);*/
        /*
            // To print each byte individually:
            print!("PNG Signature ==> ");
            for byte in &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE] {
                print!("{:02X} ", byte);
            }
            println!();
         */
         //////////////////////////////////////////////////////////////////////////////////////////////////
         // End of PNG Signature

        // Start of IHDR Chunk
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // 2. Add IHDR chunk
        // 2.1 Add IHDR length (4 bytes) - Length of the IHDR data (13 bytes)
        //let ihdr_length: u32 = constants::LENGTH_OF_IHDR_DATA; // IHDR data is always 13 bytes
        //buffer.extend_from_slice(&ihdr_length.to_be_bytes());
        buffer.extend_from_slice(&constants::LENGTH_OF_IHDR_DATA);

        // 2.2 Add IHDR type (4 bytes)
        //buffer.extend_from_slice(b"IHDR"); // Or use your constants::PNG_IHDR_TYPE_SIGNATURE
        buffer.extend_from_slice(&constants::PNG_IHDR_TYPE_SIGNATURE);

        // 2.3 Add actual IHDR data (13 bytes)
        // Width (4 bytes)
        buffer.extend_from_slice(&width.to_be_bytes());
        // Height (4 bytes)
        buffer.extend_from_slice(&height.to_be_bytes());
        // Bit depth (1 byte), rest of data for IHDR
        buffer.extend_from_slice(&constants::IHDR_DATA_FOR_UNCOMPRESSED_FILE);

        //unsafe { update_crc(0xfffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD), constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize }) };

        let mut crc: u32 = unsafe { update_crc(0xfffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } ) } ^ 0xffffffff;

        buffer.extend_from_slice(&crc.to_be_bytes());

        // It got commented
        /*println!("CRC ==========>>>>>>> ==> {:02X?}", crc);*/

        /*println!("DATA SO FAR ==========>>>>>>> ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD]);*/
        /* Endo fo IHDR Chunk creation */
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // End of IHDR Chunk
        
        // Start of IDAT Chunk
        //////////////////////////////////////////////////////////////////////////////////////////////////
        buffer.extend_from_slice(&(*inflated_data).len().to_be_bytes());  
        buffer.extend_from_slice(&constants::PNG_IDAT_TYPE_SIGNATURE);  
        //buffer.extend_from_slice(&(*inflated_data).data);        
        buffer.extend_from_slice(std::slice::from_raw_parts( (*inflated_data).data, (*inflated_data).size as usize));

        //crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add( constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD  + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } as usize + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) }) } ^ 0xffffffff;

        crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add( constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD  + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } as usize + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + (*inflated_data).size /*as usize*/) } ^ 0xffffffff;

        buffer.extend_from_slice(&crc.to_be_bytes());

        //println! ("Length of inflated data = {:02X?}", (*inflated_data).len());
        //println! ("Length of inflated data = {:02X?}", (*inflated_data).len().to_be_bytes());        
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // End of IDAT Chunk

        //println!("DATA SO FAR ==========>>>>>>> ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD]);

        //println! ("crc = {:02X?}", crc);
        
        // Start of IEND Chunk
        /////////////////////////////////////////////////////////////////////////////////////////////////

        // 1. Add IEND length (4 bytes) - Length of the IEND data (0 bytes)
        buffer.extend_from_slice(&constants::LENGTH_OF_IEND_DATA);

        // 2. Add IEND type (4 bytes)
        buffer.extend_from_slice(&constants::PNG_IEND_TYPE_SIGNATURE);

        // 3. Add actual IEND data (0 bytes)
        // No data for IEND

        // 4. Add CRC (4 bytes)

        // (constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD) + constants::LENGTH_OF_LENGTH_FIELD

        crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add((constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD) + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) }) } ^ 0xffffffff;
        //crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add( constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD  + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } as usize + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) }) } ^ 0xffffffff;
        
        //crc = unsafe { update_crc(0xffffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) }), 0x00) } ^ 0xffffffff;
        //crc = unsafe { update_crc(0xffffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD), constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) as usize }) } ^ 0xffffffff;
        buffer.extend_from_slice(&crc.to_be_bytes());   

        /////////////////////////////////////////////////////////////////////////////////////////////////
        // End of IEND Chunk

        // It got commented
        //println!("DATA SO FAR ==========>>>>>>> ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD]);

        // It got commented
        /*println! ("crc = {:02X?}", crc);*/
        
        
        // Safety: We need to properly copy the data from the raw pointer
        /* !(*inflated_data).data.is_null() {
            // Explicitly set the length of the buffer to avoid uninitialized memory
            buffer.reserve_exact(size);
            
            // Copy the data from the inflated_data pointer to our buffer
            std::ptr::copy_nonoverlapping(
                (*inflated_data).data,
                buffer.as_mut_ptr(),
                size
            );
            
            // Set the correct length after we've copied the data
            buffer.set_len(size);
        }*/

        /*println! ("ABOUT TO CREATE PNG INSTANCE......");
        
        // Write buffer data to file
        match File::create(out_put_file_path) {
            Ok(mut file) => {
                match file.write_all(&buffer) {
                    Ok(_) => println!("Successfully wrote PNG data to foo.png"),
                    Err(e) => eprintln!("Failed to write PNG data: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to create foo.png: {}", e),
        }
        
        println! ("ABOUT TO CREATE PNG INSTANCE......");*/
        // Create and return a new Png instance
        let png = Png::new(buffer);
        //png  
        
        /*png.traverse();*/

        return Some(png);
    }

    //None
}

pub fn create_png_from_boxed_defalted_data(width: u32, height: u32, inflated_data: Box<DeflatedData>, out_put_file_path: &Path) -> Option<Png> {
    unsafe {

        if ((*inflated_data).data.is_null() || (*inflated_data).len() == 0) || (width == 0 || height == 0) {

            return None;
        }

        let mut size = 0;

        //  Setting up capacity fo all PNG chunks namely PNG-Signature + IHDR + IDAT + IEND
        ////////////////////////////////////////////////////////////////////////////////////////////////
        // 1. Add PNG signature
        size = size + constants::LENGTH_OF_SIGNATURE;

        /*println! ("SIZE = {}", size);*/ // 8

        // 2. Add Chunk size for IHDR
        size = size + constants::LENGTH_OF_THREE_FIELDS;        
        size = size + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize };

        /*println! ("SIZE = {}", size); // 25 + 8 = 33*/
        
        /*println! ("-----------------------------------------------> {}",  unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize });*/


        // 3. Add Chunk size for IDAT
        size = size + constants::LENGTH_OF_THREE_FIELDS;
        size = size + (*inflated_data).size as usize;

        /*println! ("SIZE = {}, {}", size, (*inflated_data).size); // 33 + 12 +  262472 = 262517*/

        // 4. Add Chunk size for IEND
        size = size + constants::LENGTH_OF_THREE_FIELDS; // No data, length must be set to zero
        /////////////////////////////////////////////////////////////////////////////////////////////////

        /*println! ("SIZE = {}", size); // 262517 + 12 = 262529*/
        
        // Create a vector with the appropriate capacity
        let mut buffer: Vec<u8> = Vec::with_capacity(size);

        /*println! ( "Buffer = {}", buffer.len() );*/
        /*println! ( "Capacity = {}", buffer.capacity() );*/

        // Start of PNG Signature
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // 1. Add PNG signature        
        buffer.extend_from_slice(&constants::PNG_SIGNATURE);
        // Lets use the debug format specifier {:?}, to debug and print slice containing the PNG signature bytes.
        /*println!("PNG Signature ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE]);*/
        /*
            // To print each byte individually:
            print!("PNG Signature ==> ");
            for byte in &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE] {
                print!("{:02X} ", byte);
            }
            println!();
         */
         //////////////////////////////////////////////////////////////////////////////////////////////////
         // End of PNG Signature

        // Start of IHDR Chunk
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // 2. Add IHDR chunk
        // 2.1 Add IHDR length (4 bytes) - Length of the IHDR data (13 bytes)
        //let ihdr_length: u32 = constants::LENGTH_OF_IHDR_DATA; // IHDR data is always 13 bytes
        //buffer.extend_from_slice(&ihdr_length.to_be_bytes());
        buffer.extend_from_slice(&constants::LENGTH_OF_IHDR_DATA);

        // 2.2 Add IHDR type (4 bytes)
        //buffer.extend_from_slice(b"IHDR"); // Or use your constants::PNG_IHDR_TYPE_SIGNATURE
        buffer.extend_from_slice(&constants::PNG_IHDR_TYPE_SIGNATURE);

        // 2.3 Add actual IHDR data (13 bytes)
        // Width (4 bytes)
        buffer.extend_from_slice(&width.to_be_bytes());
        // Height (4 bytes)
        buffer.extend_from_slice(&height.to_be_bytes());
        // Bit depth (1 byte), rest of data for IHDR
        buffer.extend_from_slice(&constants::IHDR_DATA_FOR_UNCOMPRESSED_FILE);

        //unsafe { update_crc(0xfffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD), constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize }) };

        let mut crc: u32 = unsafe { update_crc(0xfffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } ) } ^ 0xffffffff;

        buffer.extend_from_slice(&crc.to_be_bytes());

        // It got commented
        /*println!("CRC ==========>>>>>>> ==> {:02X?}", crc);*/

        /*println!("DATA SO FAR ==========>>>>>>> ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD]);*/
        /* Endo fo IHDR Chunk creation */
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // End of IHDR Chunk
        
        // Start of IDAT Chunk
        //////////////////////////////////////////////////////////////////////////////////////////////////
        buffer.extend_from_slice(&(*inflated_data).len().to_be_bytes());  
        buffer.extend_from_slice(&constants::PNG_IDAT_TYPE_SIGNATURE);  
        //buffer.extend_from_slice(&(*inflated_data).data);        
        buffer.extend_from_slice(std::slice::from_raw_parts( (*inflated_data).data, (*inflated_data).size as usize));

        //crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add( constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD  + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } as usize + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) }) } ^ 0xffffffff;

        crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add( constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD  + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } as usize + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + (*inflated_data).size /*as usize*/) } ^ 0xffffffff;

        buffer.extend_from_slice(&crc.to_be_bytes());

        //println! ("Length of inflated data = {:02X?}", (*inflated_data).len());
        //println! ("Length of inflated data = {:02X?}", (*inflated_data).len().to_be_bytes());        
        //////////////////////////////////////////////////////////////////////////////////////////////////
        // End of IDAT Chunk

        //println!("DATA SO FAR ==========>>>>>>> ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD]);

        //println! ("crc = {:02X?}", crc);
        
        // Start of IEND Chunk
        /////////////////////////////////////////////////////////////////////////////////////////////////

        // 1. Add IEND length (4 bytes) - Length of the IEND data (0 bytes)
        buffer.extend_from_slice(&constants::LENGTH_OF_IEND_DATA);

        // 2. Add IEND type (4 bytes)
        buffer.extend_from_slice(&constants::PNG_IEND_TYPE_SIGNATURE);

        // 3. Add actual IEND data (0 bytes)
        // No data for IEND

        // 4. Add CRC (4 bytes)

        // (constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD) + constants::LENGTH_OF_LENGTH_FIELD

        crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add((constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe {big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD) + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) }) } ^ 0xffffffff;
        //crc = unsafe { update_crc (0xFFFFFFFF, buffer.as_ptr().add( constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD  + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) } as usize + constants::LENGTH_OF_LENGTH_FIELD), (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) }) } ^ 0xffffffff;
        
        //crc = unsafe { update_crc(0xffffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + (constants::LENGTH_OF_TYPE_FIELD as u32) + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) }), 0x00) } ^ 0xffffffff;
        //crc = unsafe { update_crc(0xffffffff, buffer.as_ptr().add(constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD), constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) as usize }) } ^ 0xffffffff;
        buffer.extend_from_slice(&crc.to_be_bytes());   

        /////////////////////////////////////////////////////////////////////////////////////////////////
        // End of IEND Chunk

        // It got commented
        //println!("DATA SO FAR ==========>>>>>>> ==> {:02X?}", &buffer.as_slice()[0..constants::LENGTH_OF_SIGNATURE + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IHDR_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + (*inflated_data).size as usize + constants::LENGTH_OF_CRC_FIELD + constants::LENGTH_OF_LENGTH_FIELD + constants::LENGTH_OF_TYPE_FIELD + unsafe { big_endian_read_u32(constants::LENGTH_OF_IEND_DATA.as_ptr()) as usize } + constants::LENGTH_OF_CRC_FIELD]);

        // It got commented
        /*println! ("crc = {:02X?}", crc);*/
        
        
        // Safety: We need to properly copy the data from the raw pointer
        /* !(*inflated_data).data.is_null() {
            // Explicitly set the length of the buffer to avoid uninitialized memory
            buffer.reserve_exact(size);
            
            // Copy the data from the inflated_data pointer to our buffer
            std::ptr::copy_nonoverlapping(
                (*inflated_data).data,
                buffer.as_mut_ptr(),
                size
            );
            
            // Set the correct length after we've copied the data
            buffer.set_len(size);
        }*/

        
        /*
                println! ("ABOUT TO CREATE PNG INSTANCE......");
        
                // Write buffer data to file
                match File::create(out_put_file_path) {
                    Ok(mut file) => {
                        match file.write_all(&buffer) {
                            Ok(_) => println!("Successfully wrote PNG data to foo.png"),
                            Err(e) => eprintln!("Failed to write PNG data: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Failed to create foo.png: {}", e),
                }
         */
        
        /*println! ("ABOUT TO CREATE PNG INSTANCE......");*/
        // Create and return a new Png instance
        let png = Png::new(buffer);
        //png  
        
        //png.traverse();

        return Some(png);
    }

    //None
}

/// Modifies PNG pixel data for testing and validation purposes.
///
/// This function is specifically designed to test pixel data manipulation in inflated PNG chunks.
/// It modifies pixel values based on the PNG color type and bit depth, then stores the changes
/// back into the inflated data structure. The modified data can later be written to a file
/// for physical proof that the PNG parsing and manipulation library is working correctly.
///
/// # Purpose
/// - Test pixel data access and modification
/// - Validate PNG parsing functionality  
/// - Provide physical proof of successful PNG manipulation
/// - Serve as a foundation for more complex pixel operations
///
/// # Parameters
/// * `pixels` - Mutable pointer to inflated PNG pixel data
/// * `data` - Vector containing replacement color values (RGB components)
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels  
/// * `color_type` - PNG color type (2 for RGB truecolor)
/// * `bit_depth` - Bits per color component (typically 8)
///
/// # Returns
/// * Returns the same pointer to the modified inflated data
///
/// # Safety
/// This function uses unsafe pointer operations to directly modify pixel data.
/// Caller must ensure the pixels pointer is valid and points to sufficient memory.
///
/// # Current Implementation
/// - Supports RGB truecolor (color_type=2, bit_depth=8)
/// - Processes scanline-by-scanline with filter byte handling
/// - Replaces pixel values with provided data vector values
/// - Function will be extended based on future requirements
///
/// # Example
/// ```rust
/// let modified = modify_png_pixel_data(
///     inflated_data_ptr,
///     vec![0xFF, 0x00, 0x00], // Red color
///     width,
///     height,
///     2, // RGB truecolor
///     8  // 8-bit depth
/// );
/// ```
pub fn modify_png_pixel_data (pixels: *mut InflatedData, data: Vec<u8>, width: u32, height: u32, color_type: u8, bit_depth: u8) -> *mut InflatedData {

    // Here we can modify the pixel data based on the color type and bit depth
    // For example, if color_type is 2 (Truecolor), we can modify RGB values
    // If color_type is 3 (Indexed), we can modify palette indices

    // This is a placeholder for actual pixel modification logic
    // For now, we just return the original data pointer

    // Check if we're dealing with RGB truecolor format with 8-bit depth
    // Color type 2 = RGB (3 bytes per pixel), bit depth 8 = 8 bits per color component
    if color_type == 2 && bit_depth == 8 {

        unsafe {

            // Counter to track current row being processed (for debugging purposes)
            let mut idx = 0;

            // Iterate through each row (scanline) of the image
            // PNG stores image data as horizontal scanlines from top to bottom
            for i in (0..height) {

                // Iterate through each pixel in the current scanline
                // j starts at 1 because byte 0 of each scanline is the filter byte
                // Each pixel is 3 bytes (RGB), so we step by 3
                // Total scanline length = width * 3 bytes + 1 filter byte
                /*J needs to originate at 1, becuase 0 byte of each line is always 0 in PNG file*/
                for j in (1..((width*3 + 1) as usize)).step_by(3) {

                    // Check if the filter byte (first byte of scanline) is 0x00
                    // Filter byte 0x00 means "None" - no filtering applied to this scanline
                    // Calculate scanline start: row_index * (width * 3_bytes_per_pixel + 1_filter_byte)
                    /*first byte must be 0x00 and this is working as well because idx is same as height of the image*/
                    if *(*pixels).data.add((i*(width*3 + 1)) as usize) == 0x00 /*&& *(*dat).data.add(1) == 0x00 && *(*dat).data.add(2) == 0x00*/ {

                        // Update row counter (idx will equal current row + 1)
                        // This serves as a debugging aid to verify we're processing all rows                
                        /*idx needs to originate at 1, this working perfectly*/
                        idx = i + 1;

                        // Note: The commented condition would check if current pixel is non-zero
                        // Currently we modify ALL pixels regardless of their original values
                        /*if *(*dat).data.add((i*(width + 1) + (j + 0)) as usize) != 0x00 && *(*dat).data.add((i*(width + 1) + (j + 1)) as usize) != 0x00 && *(*dat).data.add((i*(width + 1) + (j + 2)) as usize) != 0x00*/ {

                            // Modify the RGB components of the current pixel
                            // Calculate pixel address: scanline_start + pixel_offset_in_scanline

                            
                            // Set Red component to data[0] (typically 0xFF for pure red)
                            *(*pixels).data.add(((i*(width*3 + 1)) as usize + (j + 0))) = data[0] ; // R  
                            // Set Green component to data[1] (typically 0x00 for pure red)
                            *(*pixels).data.add(((i*(width*3 + 1)) as usize + (j + 1))) = data[1];  // G
                            // Set Blue component to data[2] (typically 0x00 for pure red)
                            *(*pixels).data.add(((i*(width*3 + 1)) as usize + (j + 2))) = data[2];  // B
                        }
                    }                                    
                }
            }

            // Debug output to verify we processed all rows correctly
            // idx should equal height if all scanlines were processed
            /*The idx value and the height are same which is 344 and this the actaul height of the image I have checked it.*/
            //println! ("-------------->>>>>>> idx = {}, height = {}", idx, height);

            /*Return the modified pixel data}*/

            // Note: Function completes pixel modification here
            // The modified pixel data remains in memory for later use
        }    
    }

    // Return the pointer to the modified inflated data
    // The same pointer is returned but the data it points to has been modified
    pixels
} 

