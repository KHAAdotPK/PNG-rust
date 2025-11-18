/*
    lib/rust/png/src/images.rs
    Q@khaa.pk
 */

/// Flexibility to handle both formats if needed
/// Enumeration defining supported tensor data layout formats for image processing.
/// 
/// This enum provides flexibility in handling different tensor memory layouts that may be
/// required when interfacing with various image processing libraries, GPU frameworks, or
/// legacy systems. The choice of format significantly impacts memory access patterns,
/// performance characteristics, and compatibility with different computational backends.
/// 
/// # Variants
/// 
/// ## CHW (Channels, Height, Width) - **Recommended Default**
/// - **Memory Layout**: All pixels of channel 0, then all pixels of channel 1, etc.
/// - **Use Cases**: 
///   - Convolutional neural networks and Vision Transformers
///   - GPU-accelerated operations (CUDA, OpenCL)
///   - Deep learning frameworks (PyTorch, TensorFlow)
///   - JEPA patch extraction and processing
/// - **Performance**: Optimized for channel-wise operations and SIMD instructions
/// - **Batch Dimension**: Extends to NCHW (Batch, Channels, Height, Width)
/// 
/// ## HWC (Height, Width, Channels) - **Interoperability Format**
/// - **Memory Layout**: Interleaved channels (R,G,B,R,G,B,...)
/// - **Use Cases**:
///   - OpenCV image processing operations
///   - Some image I/O libraries and codecs
///   - CPU-based image manipulation
///   - Interfacing with graphics APIs expecting interleaved data
/// - **Performance**: Better for pixel-wise operations but slower for ML computations
/// 
/// # Performance Implications
/// - **CHW**: Superior cache locality for convolutions and matrix operations
/// - **HWC**: Better for pixel-level processing but requires transposition for ML ops
/// 
/// # Example Usage
/// ```rust
/// // Using CHW for ML operations (recommended)
/// model.start_training_loop(ImageDataTensorShapeFormat::CHW);
/// 
/// // Using HWC when interfacing with OpenCV or similar libraries
/// model.start_training_loop(ImageDataTensorShapeFormat::HWC);
/// ```
/// 
/// # Implementation Notes
/// The format parameter allows the training loop to adapt its data preprocessing
/// and tensor operations based on the input format, ensuring optimal performance
/// regardless of the source data layout.
#[derive(PartialEq)]
pub enum ImageDataTensorShapeFormat {
    CHW,  // The primary choice
    HWC,  // For interfacing with certain image libraries
} 

/// Represents the dimensional structure of image data tensors.
/// 
/// This struct defines the shape characteristics of image data used in machine learning
/// models, particularly for computer vision tasks. It encapsulates the three fundamental
/// dimensions that define image data structure.
/// 
/// # Fields
/// * `channels` - The number of color channels (e.g., 3 for RGB, 1 for grayscale)
/// * `height` - The vertical dimension of the image in pixels
/// * `width` - The horizontal dimension of the image in pixels  
/// 
/// # Example
/// ```rust
/// // RGB image of 224x224 pixels
/// let shape = ImageDataTensorShape::new(3, 224, 224);
/// 
/// // Grayscale image of 28x28 pixels (like MNIST)
/// let mnist_shape = ImageDataTensorShape::new(1, 28, 28);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ImageDataTensorShape {

    channels: usize,
    height: f64,
    width: f64,    
}

/// Implementation block for ImageDataTensorShape providing constructor and accessor methods.
/// 
/// This implementation ensures controlled access to the image tensor dimensions through
/// getter methods. It provides a clean interface for retrieving shape information that
/// can be used for tensor operations and model architecture configuration.
/// 
/// # Methods
/// * `new()` - Creates a new ImageDataTensorShape with specified dimensions
/// * `get_height()` - Returns the height dimension of the image tensor
/// * `get_width()` - Returns the width dimension of the image tensor
/// * `get_channels()` - Returns the number of channels in the image tensor
impl ImageDataTensorShape {

    pub fn new(channels: usize, height: f64, width: f64) -> ImageDataTensorShape {

        ImageDataTensorShape {

            channels: channels,
            height: height,
            width: width,            
        }
    }

    pub fn get_height(&self) -> f64 {

        self.height
    }

    pub fn get_width(&self) -> f64 {

        self.width
    }

    pub fn get_channels(&self) -> usize {

        self.channels
    }
}

// The context/target block
pub struct ImageBlock {
    
    height: f64,
    width: f64,
    size: usize    
}

impl ImageBlock {

    pub fn new(height: f64, width: f64, size: usize) -> ImageBlock {

        ImageBlock {

            height: height,
            width: width,
            size: size,            
        }        
    }

    pub fn get_height(&self) -> f64 {

        self.height
    }

    pub fn get_width(&self) -> f64 {

        self.width
    }

    pub fn get_size(&self) -> usize {

        //self.size
        //((self.get_width() *self.get_height()) as usize)

        (self.get_width() as usize)*(self.get_height() as usize)
    }
}

