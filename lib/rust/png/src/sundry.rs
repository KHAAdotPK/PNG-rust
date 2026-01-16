/*
   lib/rust/png/src/sundry.rs
   Q@khaa.pk
*/

#[macro_export]
macro_rules! input_pipeline_slice_start {
    ($image_slice_height: expr, $image_slice_width: expr, $image_channels: expr, $image_number: expr) => {
        (($image_channels as f64)
            * $image_slice_height
            * $image_slice_width
            * ($image_number as f64))
    };
}

#[macro_export]
macro_rules! input_pipeline_slice_end {
    ($image_slice_height: expr, $image_slice_width: expr, $image_channels: expr, $image_number: expr) => {
        (($image_channels as f64)
            * $image_slice_height
            * $image_slice_width
            * (($image_number + 1) as f64))
    };
}

#[macro_export]
macro_rules! image_block_slice_start_experimental {

    ($block_number: expr, $image_dims: expr, $block_dims: expr) => {

        let image_height = $image_dims.get_height();
        let image_width = $image_dims.get_width();
        //let image_channels = $image_dims.get_channels();
        let block_height = $block_dims.get_height();
        let block_width = $block_dims.get_width();
        //let block_channels = $block_dims.get_channels();

        println! ("block_width = {}", block_width);
        println! ("block_height = {}", block_height);
        println! ("image_width = {}", image_width);
        println! ("image_height = {}", image_height);



        //let block_line_index = (($block_number - 1)) / ((image_width as usize)/(block_width as usize);


        // 1. CALCULATE GRID DIMENSIONS
        // Calculate the number of blocks that fit in a single horizontal line (floor division).
        let mut number_of_blocks_per_line: usize = (image_width / block_width) as usize;


        // Calculate remaining pixels at the end of the line that don't form a full block.
        let non_overlapping_pixels_per_line: usize =  (image_width as usize) - (number_of_blocks_per_line * (block_width as usize));


        // If there is a remainder (partial block), increment the block count
        // to ensure the full width of the image is covered (Ceiling Division).
        if non_overlapping_pixels_per_line > 0 {

            number_of_blocks_per_line = number_of_blocks_per_line + 1;
        }

        // 2. CONVERT LINEAR BLOCK ID TO 2D COORDINATES
        // Calculate the vertical Row Index (0-based).
        // We subtract 1 from $block_number to handle 1-based input indexing.
        let mut block_line_number: usize = (($block_number - 1) as usize) / number_of_blocks_per_line;

        // Initialize the horizontal Column Index.
        let mut block_index: usize = $block_number as usize;

        // If the block number exceeds the first line, calculate its offset relative to its current row.
        // Formula: Current_ID - (Current_Row * Width)
        if ($block_number as usize) > number_of_blocks_per_line {

            block_index = ((($block_number - 0) as usize) - ((block_line_number + 0) * number_of_blocks_per_line));
        }

        // Convert the Column Index from 1-based to 0-based.
        if block_index > 0 {

            block_index = block_index - 1;
        }


        // 3. DEBUG OUTPUT
        println! ("number_of_blocks_per_line -> {}", number_of_blocks_per_line);
        println! ("non_overlapping_pixels_per_line -> {}", non_overlapping_pixels_per_line);
        println! ("block_line_number = {}", block_line_number);
        println! ("block_index = {}", block_index);




        println! ("($block_number * block_width) = {}", (($block_number as usize) * (block_width as usize)));

        println!("image_width / block_width = {}", (image_width as usize)/(block_width as usize));

        println! ("->>>>>>> >>>>> {} >>>>>> >>>>>> {}", ($block_number - 0), (($block_number - 0) as usize) / ((image_width as usize)/(block_width as usize)));



        ($block_number -1)
    }
}

#[macro_export]
macro_rules! image_block_slice_start {
    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {
        ((($block_number - 1) as usize)
            * ($image_block_width as usize)
            * ($image_channels as usize)) as f64
    };
}

#[macro_export]
macro_rules! image_block_slice_end {
    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {
        ((($block_number as usize) * ($image_block_width as usize) * ($image_channels as usize))
            - 0) as f64
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
        (($input_len / $channels) as f64
            / (JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS) as f64
            / JEPA_IMAGES_ASPECT_RATIO)
            .sqrt() as f64
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
        (($input_len / $channels) as f64
            / (JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS) as f64
            / JEPA_IMAGES_ASPECT_RATIO)
            .sqrt() as f64
            * JEPA_IMAGES_ASPECT_RATIO
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
        ($input_len / $channels) / (JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS)
    };
}

/* **************************************************************************** */
#[macro_export]
macro_rules! image_block_slice_start_vertical {
    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {
        ((($block_number - 1) as usize)
            * ($image_block_width as usize)
            * ($image_channels as usize)) as f64
    };
}

/*
   CONVERT LINEAR BLOCK ID TO 2D COORDINATES (VERTICAL)
   Random block number to line number (image stride)
*/
#[macro_export]
macro_rules! image_block_slice_start_vertical_experimental {
    ($block_number: expr, $channels: expr, $image_dims: expr, $block_dims: expr) => {
        ((($block_number - 0) as usize)
            / ($image_dims.get_width() / ($block_dims.get_width())) as usize)
            /*/ (($image_dims.get_width()) as usize / ($block_dims.get_width() as usize))*/
            * ($block_dims.get_height()) as usize
    };
}

#[macro_export]
macro_rules! image_block_slice_end_vertical {
    ($block_number: expr, $image_block_width: expr, $image_channels: expr) => {
        ((($block_number as usize) * ($image_block_width as usize) * ($image_channels as usize))
            - 0) as f64
    };
}

#[macro_export]
macro_rules! image_block_width_vertical {
    ($image_tensor: expr) => {
        $image_tensor.get_width()
            / ((JEPA_NUMBER_OF_CONTEXT_BLOCKS + JEPA_NUMBER_OF_TARGET_BLOCKS) as f64)
    };
}

#[macro_export]
macro_rules! image_block_height_vertical {
    ($image_tensor: expr) => {
        $image_tensor.get_height()
    };
}

#[macro_export]
macro_rules! image_block_size_vertical {
    ($image_tensor: expr) => {
        image_block_width_vertical!($image_tensor) * image_block_height_vertical!($image_tensor)
    };
}
