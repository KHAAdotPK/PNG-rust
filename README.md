# PNG-rust
A Rust crate to create and edit PNG files. 

## Integration as a Dependency
To use PNG-rust in your project, follow these steps:

1. **Clone the Repository**: First, clone the PNG-rust repository into your project's `lib` directory:
```bash
# From your project root directory
mkdir -p lib
cd lib
git clone https://github.com/KHAAdotPK/PNG-rust.git
```

2. **Build zlib Dependency**: Run the included PULL.cmd script to clone and build zlib:
```bash
cd lib/PNG-rust
PULL.cmd
```
This will clone the zlib repository and build it with CMake, creating the necessary Debug build files.

3. **Add as Dependency**: In your project'sCargo.toml file, add the following dependency:
```toml
[dependencies]
png = { path = "./lib/PNG-rust/lib/rust/png/" }
```

### Example Usage: 
```RUST
/*
    src/main.rs
    Q@khaa.pk
 */

use std::{str, path::Path, fs::{File, metadata}, io::Read}; 
use png::{Png, Chunk}; 

#[link(name = "png", kind = "dylib")]
extern {
 
    fn big_endian_read_u32(ptr: *const u8) -> u32;     
}

fn main() {

    let mut i: usize = 0;
    let mut buffer: Vec<u8>;
    let path = Path::new("assets/tmfinr.png");
    /*
        The file will be closed once the scope of its owner ends. 
        If you need it to live for less time, you can introduce a new scope where it will live.
        If you need it to live for more time, you can move the ownership of the file to a new owner.
     */
    let file = File::open(&path);

    match file {

        Err (why) => {

            panic!("Couldn't open {}: {}", path.display().to_string(), why);    
        }

        Ok (mut f) => {

            buffer = vec![0; metadata("assets/tmfinr.png").unwrap().len() as usize];

            f.read (&mut buffer).unwrap();

            /*    
                The idiomatic way to control how long it's open is to use a scope { }.
                The file will be automatically dropped when the "scope" is done (this is usually when a function exits).
                There's one other way to manually close the file, using the drop() function. The drop() function does the exact same thing as what happens when the scope around the file closes. 
             */
            drop(f);                        
        }
    }

    let png = Png::new(buffer);
    let mut iter = png.chunks.iter();

    println!("Number of chunks = {}", png.chunks.len());
    loop {

        if !(i < png.chunks.len()) {

            break;
        }

        let chunk: &Chunk = iter.next().unwrap();

        println! ("Length = {}",  unsafe { big_endian_read_u32 (chunk.length.clone().as_mut_ptr()) });        
        println! ("Type = [ {} {} {} {} ], {}", chunk.type_name[0], chunk.type_name[1], chunk.type_name[2], chunk.type_name[3], str::from_utf8(&chunk.type_name).unwrap());
                
        i = i + 1;        
    }
}
```

### Another bit larger example of usage: 
```RUST
/*
 * src/main.rs
 * Q@khaa.pk
 */

#![allow(non_snake_case)]

use std::{cell::RefCell, fs::{File, metadata}, io::Read, io::Write, path::Path, path::PathBuf, rc::Rc, str};
use argsv::{common_argc, find_arg, help, help_line, process_argument, start, stop, COMMANDLINES, PCLA};
use numrs::{dimensions::Dimensions, collective::Collective, num::Numrs};
use png::{constants, Png, Chunk, DeflatedData, InflatedData, create_uncompressed_png, modify_png_pixel_data}; 

use jepa::images::{Model, ModelConfig, ImageDataTensorShape, ImageDataTensorShapeFormat};

fn main() {

    let command_lines = "h -h help --help ? /? (Displays help screen)\n\
                         v -v version --version /v (Displays version number)\n\
                         t -t traverse --traverse /t (Traverses PNG file structure and displays it)\n\
                         d -d delete --delete /d (Deletes the named chunk from the PNG file)\n\
                         verbose --verbose (Displays detailed information or feedback about the execution of another command)\n\
                         suffix --suffix (Suffix for the output PNG file)\n";

    let arg_suffix: *mut COMMANDLINES;

    let mut suffix_token: Option<&str> = Some(constants::PNG_OUTPUT_FILE_SUFFIX);                 

    // Get the command-line arguments as an iterator
    let args: Vec<String> = std::env::args().collect();
    // Create a Vec<CString> from the Vec<String>
    let c_args: Vec<std::ffi::CString> = args.iter().map(|s| std::ffi::CString::new(s.as_str()).expect("Failed to create CString")).collect();
    // Get the equivalent of argv in C. Create a Vec<*const c_char> from the Vec<CString>.
    let c_argv: Vec<*const std::os::raw::c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
    // Get the C equivalent of argc.    
    let argc: i32 = c_args.len() as std::os::raw::c_int;

    let mut ncommon: i32 = 0;

    let head = start (argc, c_argv.clone(), command_lines);
        
        ncommon = common_argc (head);

        let arg_help = find_arg (head, command_lines, "h");
        if !arg_help.is_null() || argc < 1 {
            
            help (head, command_lines);
            stop (head); 

            return;
        }
        
        arg_suffix = find_arg (head, command_lines, "--suffix");
        if !arg_suffix.is_null() {

            let arg_suffix_clap: *mut PCLA = unsafe {(*arg_suffix).get_clap()};
            if unsafe{(*arg_suffix_clap).get_argc()} > 0 {

                suffix_token = Some(unsafe { str::from_utf8_unchecked(&args[unsafe{(*arg_suffix_clap).get_index_number() as usize} + 1].as_bytes()) });
             } 
        } 
        
    stop (head); 
           
    // for loop with range
    for i in 1..ncommon {

        let arg = &args[i as usize];

        let path: &Path = Path::new(arg);

        let mut height: u32 = 0;
        let mut width: u32 = 0;

        let mut color_type: u8 = 0;
        let mut bit_depth: u8 = 0;

        // Check if the file exists and has a PNG extension
        if path.exists() && path.extension().map_or(false, |ext| ext == "png") {

            println!("Processing PNG file: {}", arg);
                               
            /*
             * The file will be closed automatically when `file` goes out of scope.
             * If needed, you can limit its lifetime by introducing a new block scope.
             */
            let file = File::open(&path);
            
            match file {

                Err (why) => {
                            
                    // Skip problematic files instead of panicking
                    println!("Skipping file: {}, couldn't open because of {}.", path.display().to_string(), why);
                    continue;  // Move to the next file in the loop
                }
                                        
                Ok (mut f) => {
                    
                    /*
                     * Reads the entire file into a pre-allocated buffer.
                     * 
                     * - Uses `path.metadata()`.
                     * - Handles potential errors explicitly instead of unwrapping.
                     * - Buffer size matches the file length (in bytes).
                     */                    
                    let file_size = match path.metadata() {
                        Ok(meta) => meta.len() as usize,
                        Err(e) => {
                            println!("Failed to read metadata for {}: {}", path.display(), e);
                           
                            // About `drop()`:
                            // - NOT NEEDED HERE because:
                            //   1. `continue` automatically triggers Rust's destructors (including file closing)                            
                            //   2. Rust's RAII guarantees cleanup when variables go out of scope
                            //
                            // When WOULD you need `drop()`?
                            // - To force early release of resources (e.g., locks before blocking ops)
                            // - When explicit cleanup timing matters (e.g., temp files)
                            // - When breaking circular references (rare in Rust)

                            continue; // Skip to next file
                        }
                    };
                    
                    let mut buffer = vec![0; file_size];
        
                    f.read (&mut buffer).unwrap();

                    // Read file contents into the buffer
                    if let Err(e) = f.read(&mut buffer) {
                        println!("Failed to read file {}: {}", path.display(), e);

                        // About `drop()`:
                        // - NOT NEEDED HERE because:
                        //   1. `continue` automatically triggers Rust's destructors (including file closing)                            
                        //   2. Rust's RAII guarantees cleanup when variables go out of scope
                        //
                        // When WOULD you need `drop()`?
                        // - To force early release of resources (e.g., locks before blocking ops)
                        // - When explicit cleanup timing matters (e.g., temp files)
                        // - When breaking circular references (rare in Rust)

                        continue;
                    }
        
                    /*    
                        The idiomatic way to control how long it's open is to use a scope { }.
                        The file will be automatically dropped when the "scope" is done (this is usually when a function exits).
                        There's one other way to manually close the file, using the drop() function. The drop() function does the exact same thing as what happens when the scope around the file closes. 
                     */
                    drop(f); 
                    
                    let png = Png::new(buffer);

                    match png.match_color_type_and_bit_depth(2, 8) {
                                                
                        false => {

                            println!("Skipping file: {}, it has unsupported color type/bit depth combination.", path.display().to_string());
                            continue; // Skip to next file in the loop    
                        },
                        _ => {

                        }                        
                    }

                    let chunk: Option<&Chunk> = png.get_chunk_by_type("IHDR");

                    match chunk {

                        Some (chunk) => {

                            height = chunk.get_height();
                            width = chunk.get_width();

                            color_type = chunk.get_color_type();
                            bit_depth = chunk.get_bit_depth();
                        }
                        None => {

                        }
                    }

                    //let all_idat_data_deflated: *mut DeflatedData = png.get_all_idat_data_as_DeflatedData();

                    /*
                        Concatenate → Inflate
                        PNG IDAT Chunks Are Fragments of a Single Zlib Stream
                        The PNG spec treats all IDAT chunks as parts of one continuous compressed stream
                        Concatenating them first is required for correct decompression
                        (get_all_idat_data_as_vec() → get_inflated_data()) follows the standard.
                     */
                    let all_idat_data: Vec<u8> = png.get_all_idat_data_as_vec(); // Combine raw IDAT chunks
                    let mut dat: *mut InflatedData = png.get_inflated_data(&all_idat_data); // Inflate the combined data all at once

                    /*
                        Modifying pixels after inflation  
                    */
                    dat = modify_png_pixel_data(dat, Vec::from([0xFF, 0x00, 0x00]), width, height, color_type, bit_depth);

                    /*
                        The Box now owns the memory pointed to by dat.
                        The Box is a smart pointer that manages the memory of its contents.                        
                        If boxed_dat goes out of scope without being passed further, Drop will free the memory
                    */
                    let mut boxed_dat: Box<DeflatedData>;
                    unsafe { 
                        boxed_dat = Box::from_raw(dat); 

                        // Just to show that the memory is managed by the Box and no dereferencing is needed to access the data
                        //println!("Length of boxed_dat = {}", boxed_dat.len());

                        /*
                            Memory cleanup:
                            If you're done with boxed_dat completely, you can drop it explicitly:
                            drop(boxed_dat);
                            This will automatically call the drop implementation for DeflatedData and deallocate the memory.
                        */
                        //drop(boxed_dat); // Commented out because it is implicitly dropped when the scope ends
                    };
                    
                    /*
                        Deflate the entire pixel data in one go. 
                        Split the result into IDAT chunks (optional and not implemented yet, but some encoders do this for streaming).
                    */
                    //let deflated_data: *mut DeflatedData = png.get_deflated_data(dat);
                    /*
                        Ownership Transfer in get_deflated_data_new
                        When you pass a Box<InflatedData> to get_deflated_data, ownership is transferred to the function.
                        The Box and its contents will be dropped (freed) at the end of the function call, not at the end of main() or the outer scope.                        
                     */
                    let deflated_data: *mut DeflatedData = png.get_deflated_data_from_boxed_inflated_data (boxed_dat);
                                        
                    let output_path = path.with_extension("").with_extension(&format!("{}.png", suffix_token.unwrap()));

                    println!("Output PNG file will be: {}", output_path.display());
                    
                    create_uncompressed_png(width, height, deflated_data as *mut InflatedData, &output_path);
                }
            }

        } else {

            println!("Invalid or non-existent PNG file: {}", arg);
        }                    
    }
} 
```

Feel free to contribute and report issues.

### License
This project is governed by a license, the details of which can be located in the accompanying file named 'LICENSE.' Please refer to this file for comprehensive information.
