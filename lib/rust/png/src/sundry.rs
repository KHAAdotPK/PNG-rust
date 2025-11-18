/*
    lib/rust/png/src/sundry.rs
    Q@khaa.pk
 */

#[macro_export]
macro_rules! input_pipeline_slice_start {
    ($image_slice_height: expr, $image_slice_width: expr, $image_channels: expr, $image_number: expr) => {

        (($image_channels as f64)*$image_slice_height*$image_slice_width*($image_number as f64))
    }
}

#[macro_export]
macro_rules! input_pipeline_slice_end {
    ($image_slice_height: expr, $image_slice_width: expr, $image_channels: expr, $image_number: expr) => {

        (($image_channels as f64)*$image_slice_height*$image_slice_width*((($image_number + 1) as f64)))
    }
}

#[macro_export]
macro_rules! image_block_slice_start {

    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {
        
        ((($block_number - 1) as usize)*($image_block_width as usize)*($image_channels as usize)) as f64
    };    
}

#[macro_export]
macro_rules! image_block_slice_end {

    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {

        ((($block_number as usize)*($image_block_width as usize)*($image_channels as usize)) - 0) as f64
    };    
}

/// Calculates the height of an image block based on input dimensions and JEPA configuration.
///
/// This macro computes the height of individual image blocks by:
/// 1. Calculating total pixels per image: `input_len / channels`
/// 2. Dividing by total blocks (context + target) to get pixels per block
/// 3. Adjusting for aspect ratio to distribute pixels between height and width
/// 4. Taking the square root to convert from area to linear dimension
///
/// # Arguments
/// * `$input_len` - Total length of input data (number of elements)
/// * `$channels` - Number of color channels in the image
///
/// # Returns
/// * `f64` - Calculated height of each image block
///
/// # Formula
/// `sqrt( (total_pixels / total_blocks) / aspect_ratio )`
///
/// # Example
/// ```
/// let height = image_block_height!(1200, 3);
/// ```
// Macro annotated with `#[macro_export]` will be exported at the root of the crate instead of the module where it is defined
#[macro_export]
macro_rules! image_block_height {

    ($input_len: expr, $channels: expr) => {
                
        (($input_len/$channels) as f64/((JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS)) as f64 / JEPA_IMAGES_ASPECT_RATIO).sqrt() as f64
    };
}

/// Calculates the width of an image block based on input dimensions and JEPA configuration.
///
/// This macro computes the width of individual image blocks by:
/// 1. Calculating total pixels per image: `input_len / channels`
/// 2. Dividing by total blocks (context + target) to get pixels per block
/// 3. Adjusting for aspect ratio to distribute pixels between height and width
/// 4. Taking the square root to convert from area to linear dimension
///
/// # Arguments
/// * `$input_len` - Total length of input data (number of elements)
/// * `$channels` - Number of color channels in the image
///
/// # Returns
/// * `f64` - Calculated width of each image block
///
/// # Formula
/// `sqrt( (total_pixels / total_blocks) / aspect_ratio ) * aspect_ratio`
///
/// # Example
/// ```
/// let width = image_block_width!(1200, 3);
/// ```
// Macro annotated with `#[macro_export]` will be exported at the root of the crate instead of the module where it is defined
#[macro_export]
macro_rules! image_block_width {

    ($input_len: expr, $channels: expr) => {
                
        (($input_len/$channels) as f64/((JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS)) as f64 / JEPA_IMAGES_ASPECT_RATIO).sqrt() as f64 * JEPA_IMAGES_ASPECT_RATIO
    };
}

/// Calculates the size of an image block based on input dimensions and JEPA configuration.
///
/// This macro computes the size of individual image blocks by:
/// 1. Calculating total pixels per image: `input_len / channels`
/// 2. Dividing by total blocks (context + target) to get pixels per block
///
/// # Arguments
/// * `$input_len` - Total length of input data (number of elements)
/// * `$channels` - Number of color channels in the image
///
/// # Returns
/// * `usize` - Calculated size of each image block
///
/// # Example
/// ```
/// let size = image_block_size!(1200, 3);
/// ```
// Macro annotated with `#[macro_export]` will be exported at the root of the crate instead of the module where it is defined
#[macro_export]
macro_rules! image_block_size {

    ($input_len: expr, $channels: expr) => {
                        
        ($input_len/$channels)/(JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS)
    };
}

/* **************************************************************************** */
#[macro_export]
macro_rules! image_block_slice_start_vertical {

    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {
        
        ((($block_number - 1) as usize)*($image_block_width as usize)*($image_channels as usize)) as f64
    };    
}

#[macro_export]
macro_rules! image_block_slice_end_vertical {

    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {

        ((($block_number as usize)*($image_block_width as usize)*($image_channels as usize)) - 0) as f64
    };    
}

#[macro_export]
macro_rules! image_block_width_vertical {

    ($image_tensor: expr) => {

        $image_tensor.get_width() / ((JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS) as f64)
    }
}

#[macro_export]
macro_rules! image_block_height_vertical {

    ($image_tensor: expr) => {

        $image_tensor.get_height() 
    }
}

#[macro_export]
macro_rules! image_block_size_vertical {

    ($image_tensor: expr) => {
                        
        image_block_width_vertical!($image_tensor) * image_block_height_vertical!($image_tensor)
    };
}
