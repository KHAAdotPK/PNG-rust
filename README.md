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

Feel free to contribute and report issues.

### License
This project is governed by a license, the details of which can be located in the accompanying file named 'LICENSE.' Please refer to this file for comprehensive information.
