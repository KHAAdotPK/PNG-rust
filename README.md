# PNG-rust
A Rust crate to create and edit PNG files. 

## Build Instructions

Follow these steps to build `PNG-rust`:

1. **CMake Build:** Execute CMake on the `CMakeLists.txt` file to build the required libraries (`sundry.lib`, `sundry.dll`). 

    ```bash
    cmake ./CMakeLists.txt
    ```

2. **Copy Libraries:** Copy the newly built files (`sundry.lib`, `sundry.dll`) into the root directory (`./`).

3. **Cargo Build:** Finally, execute `cargo build` to build your crate which uses `PNG-rust`. 

    ```bash
    cargo build
    ```
Please note that the build process is still a work in progress and will be continuously improved for a more seamless experience.

Feel free to contribute and report issues.

### Example Usage: 
```RUST
/*
    src/main.rs
    Q@khaa.pk
 */

use std::{str, path::Path, fs::{File, metadata}, io::Read}; 
use png::{Png, Chunk}; 

#[link(name = "sundry", kind = "dylib")]
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

### License
This project is governed by a license, the details of which can be located in the accompanying file named 'LICENSE.' Please refer to this file for comprehensive information.
