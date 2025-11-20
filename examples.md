## Using [PNG-rust](https://github.com/KHAAdotPK/PNG-rust)

### Example 1: Reading and Parsing PNG Files

This example demonstrates how to read a PNG file from disk, parse its metadata, and extract important image properties like dimensions, color type, and bit depth.

```rust
let path = Path::new("Q.png");

/*
    The file will be closed automatically when `file` goes out of scope.
    If needed, you can limit its lifetime by introducing a new block scope.
 */
let file = File::open(&path);
            
match file {

    Err (why) => {}
                                        
    Ok (mut f) => {
                    
        /*
            Reads the entire file into a pre-allocated buffer. 
            - Uses `path.metadata()`.
            - Handles potential errors explicitly instead of unwrapping.
            - Buffer size matches the file length (in bytes).
        */                    
        let file_size = match path.metadata() {

            Ok(meta) => meta.len() as usize,
            Err(e) => { 

                // About `drop()`:
                // - NOT NEEDED HERE because:
                //   1. `continue` automatically triggers Rust's destructors (including file closing)                            
                //   2. Rust's RAII guarantees cleanup when variables go out of scope
                //
                // When WOULD you need `drop()`?
                // - To force early release of resources (e.g., locks before blocking ops)
                // - When explicit cleanup timing matters (e.g., temp files)
                // - When breaking circular references (rare in Rust)                
            }
        };
                    
        let mut buffer = vec![0; file_size]; // Buffer to store file contents, buffer size matches the file size

        // Read file contents into the buffer
        if let Err(e) = f.read(&mut buffer) {
      
            // About `drop()`:
            // - NOT NEEDED HERE because:
            //   1. `continue` automatically triggers Rust's destructors (including file closing)                            
            //   2. Rust's RAII guarantees cleanup when variables go out of scope
            //
            // When WOULD you need `drop()`?
            // - To force early release of resources (e.g., locks before blocking ops)
            // - When explicit cleanup timing matters (e.g., temp files)
            // - When breaking circular references (rare in Rust)
        }
        
        /*    
            The idiomatic way to control how long it's open is to use a scope { }.
            The file will be automatically dropped when the "scope" is done (this is usually when a function exits).
            There's one other way to manually close the file, using the drop() function. The drop() function does the exact same thing as what happens when the scope around the file closes. 
         */
        drop(f); 

        let mut height: u32 = 0;
        let mut width: u32 = 0;
        let mut color_type: u8 = 0;
        let mut bit_depth: u8 = 0;
                                        
        let png = Png::new(buffer);

        let chunk: Option<&Chunk> = png.get_chunk_by_type(PNG_IHDR_CHUNK);

        match chunk {

            Some (chunk) => {

                height = chunk.get_height();
                width = chunk.get_width();

                color_type = chunk.get_color_type();
                bit_depth = chunk.get_bit_depth();
            }
            None => {}
        }        
    }    
```

### Example 1.1: Color Type and Bit Depth Validation

This example shows how to validate that a PNG file matches specific color type and bit depth requirements using the library's matching functionality.

```rust
    match png.match_color_type_and_bit_depth(JEPA_IMAGE_COLOR_TYPE, JEPA_IMAGE_BIT_DEPTH) {
                                                
        false => {},
        _ => {}                        
    }
```

### Example 2: PNG Data Processing: 

This example demonstrates how to work with PNG image data by decompressing the IDAT chunks, concatenating them together, modifying pixel data, and then recompressing to create a new PNG file.

```rust
/*
    Concatenate → Inflate
    PNG IDAT Chunks Are Fragments of a Single Zlib Stream
    The PNG spec treats all IDAT chunks as parts of one continuous compressed stream
    Concatenating them first is required for correct decompression
    (get_all_idat_data_as_vec() → get_inflated_data()) follows the standard.
 */
let all_idat_data: Vec<u8> = png.get_all_idat_data_as_vec(); // Combine raw IDAT chunks
let mut dat: *mut InflatedData = png.get_inflated_data(&all_idat_data); // Inflate the combined data all at once

// Modifying pixels after inflation  
 dat = modify_png_pixel_data(dat, Vec::from([0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF]), width, height, color_type, bit_depth);

/*
    The Box now owns the memory pointed to by dat.
    The Box is a smart pointer that manages the memory of its contents.                        
    If boxed_dat goes out of scope without being passed further, Drop will free the memory
 */
let mut boxed_dat: Box<DeflatedData>;

unsafe { 

    boxed_dat = Box::from_raw(dat); 
}

/*
    Ownership Transfer in get_deflated_data_new
    When you pass a Box<InflatedData> to get_deflated_data, ownership is transferred to the function.
    The Box and its contents will be dropped (freed) at the end of the function call, not at the end of main() or the outer scope.                        
 */
let deflated_data: *mut DeflatedData = png.get_deflated_data_from_boxed_inflated_data (boxed_dat);

unsafe {                        
    boxed_deflated_data = Box::from_raw(deflated_data);
}
                                        
let output_path = path.with_extension("").with_extension(&format!("{}.png", suffix_token.unwrap()));

println!("Output PNG file will be: {}", output_path.display());
                    
let png_from_boxed_deflated_data: Option<Png> = create_png_from_boxed_defalted_data(width, height, boxed_deflated_data, &output_path);

match png_from_boxed_deflated_data {
    Some(png) => {

        png.traverse();

        println!("Saving PNG file: {}", output_path.display());

        png.save_to_file(&output_path);                            
    },
    None => {

        println!("Failed to create PNG from boxed deflated data");
    }
}
```

### Example 3: Removing Filter Method Byte

This example shows how to remove the filter method byte from the inflated data, at the moment it only works for filter method 0 (none).

```rust
/*
    Concatenate → Inflate
    PNG IDAT Chunks Are Fragments of a Single Zlib Stream
    The PNG spec treats all IDAT chunks as parts of one continuous compressed stream
    Concatenating them first is required for correct decompression
    (get_all_idat_data_as_vec() → get_inflated_data()) follows the standard.
*/
let all_idat_data: Vec<u8> = png.get_all_idat_data_as_vec(); // Combine raw IDAT chunks
let mut dat: *mut InflatedData = png.get_inflated_data(&all_idat_data); // Inflate the combined data all at once

// Remove filter method byte from inflated data, at the moment it only works for filter method 0 (none)
let dat_without_filter_method_byte: *mut InflatedData = png.remove_filter_bytes_from_inflated_data(dat);
```

### Example 4: Batch PNG Generation from Tensor Data

This example demonstrates how to generate multiple PNG files in batch from tensor data, creating individual image files for each item in a machine learning batch.

```rust
let dims_image = Box::new(Dimensions::new(image_data_tensor_shape.get_width(), image_data_tensor_shape.get_height()));

for i in 0..model_config.get_batch_size() {

    // Automatically dropped at end of each loop iteration            
    let input_pipeline_slice: Box<Collective<T>> = input_pipeline.get_slice (                
            input_pipeline_slice_start! (image_data_tensor_shape.get_height(), image_data_tensor_shape.get_width(), image_data_tensor_shape.get_channels(), i),
                
            input_pipeline_slice_end! (image_data_tensor_shape.get_height(), image_data_tensor_shape.get_width(), image_data_tensor_shape.get_channels(), i),

            &dims_image,
            Axis::None
    );

    let mut path_text = format!("{}{}", i + 1, PNG_FILE_EXTENSION);
    let mut path = Path::new(&path_text); 

    let mut png = create_png_from_collective::<T> (&input_pipeline_slice, &path);
            
    match png {

        Some(png) => {
        
            png.save_to_file(&path);
                            
        },
        None => {}
    }
}
```

### Example 5: Creating Context/Target Blocks

This example demonstrates how to extract specific sub-blocks (crops) from the tensor data.

To understand how we extract a rectangular block (e.g., $90 \times 120$) from a larger image (e.g., $254 \times 344$), we must first understand how the data is laid out in memory.

#### The Mental Model: 2D Images in 1D Memory

**1. The "Snake" of Data**

Imagine the `input_pipeline` buffer not as a stack of 2D images, but as one very long, continuous tape or "snake" of data. The images are flattened row-by-row and placed back-to-back.

  * **Width:** Pixels per row (e.g., 254).
  * **Height:** Number of rows (e.g., 344).
  * **Channels:** Data points per pixel (e.g., 3 for RGB).

**2. Calculating the Index**

For a pipeline containing a batch of 2 images (254x344, True Color RGB), the buffer size is calculated as:
$$254 \times 344 \times 3 \text{ (channels)} \times 2 \text{ (images)} = 524,288 \text{ bytes}$$

**3. The Layout Mapping**

Because the data is linear (Row-Major), "neighbors" in the buffer are neighbors horizontally, not vertically.

  * **Indices 0 to 253:** This is **Row 0** of Image 1.
  * **Indices 254 to 507:** This is **Row 1** of Image 1.
  * ...
  * **Index $N$:** The last pixel of Image 1.
  * **Index $N+1$:** Immediately starts **Row 0** of **Image 2**.

#### The Challenge: Strided Slicing

This linear layout creates a challenge when extracting a rectangular block.

If you want to cut out a $90 \times 120$ block from the top-left corner:

1.  You cannot simply slice the first $90 \times 120 = 10,800$ pixels.
2.  Doing so would give you Row 0 (width 254), then Row 1 (width 254), etc., until you reached 10,800 pixels. This results in a sheared, scrambled image.

**The Solution:**
To extract the block correctly, we must read 90 pixels, **skip** the remaining pixels of the image width (the stride), and then read the next 90 pixels.


```rust
// Conceptual Logic for Strided Access
let image_width = 254;
let block_width = 90;
let stride = image_width - block_width; // The data we must skip

// 1. Read 90 pixels (Row 0 of block)
// 2. Skip 'stride' pixels
// 3. Read 90 pixels (Row 1 of block)
// ... Repeat for block height
```




