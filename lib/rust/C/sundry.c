/*
    lib/rust/C/sundry.c
    Q@khaa.pk
 */

#include "sundry.h"

// Forward declaration for the dynamic version
INFLATED_DATA_PTR in_flate_dynamic(unsigned char* compressed_data, unsigned long compressed_size);

EXPORT_IMPORT INFLATED_DATA_PTR in_flate(unsigned char* compressed_data, unsigned long compressed_size) {
    // Validate input
    if (!compressed_data || compressed_size == 0) {
        printf("Invalid input: NULL data or zero size\n");
        return NULL;
    }

    // Initialize variables
    z_stream strm;
    memset(&strm, 0, sizeof(strm));
    
    // Initial output buffer size (adjust if needed)
    unsigned long initial_size = compressed_size * 2;
    unsigned char* out_buffer = (unsigned char*)malloc(initial_size);
    if (!out_buffer) {
        printf("Failed to allocate initial output buffer\n");
        return NULL;
    }

    // Initialize zlib
    strm.zalloc = Z_NULL;
    strm.zfree = Z_NULL;
    strm.opaque = Z_NULL;
    strm.avail_in = compressed_size;
    strm.next_in = compressed_data;
    strm.avail_out = initial_size;
    strm.next_out = out_buffer;

    int ret = inflateInit(&strm);
    if (ret != Z_OK) {
        printf("inflateInit failed: %s\n", strm.msg ? strm.msg : "unknown error");
        free(out_buffer);
        return NULL;
    }

    // Single-call decompression with fixed buffer
    ret = inflate(&strm, Z_FINISH);
    
    // Check if our buffer was too small
    if (ret == Z_BUF_ERROR) {
        // Clean up the first attempt
        inflateEnd(&strm);
        free(out_buffer);
        
        // Try again with dynamic buffer approach
        return in_flate_dynamic(compressed_data, compressed_size);
    }
    
    if (ret != Z_STREAM_END) {
        printf("inflate failed with code %d: %s\n", ret, strm.msg ? strm.msg : "unknown error");
        inflateEnd(&strm);
        free(out_buffer);
        return NULL;
    }

    // Calculate actual size used
    unsigned long actual_size = strm.total_out;
    
    // Clean up zlib
    inflateEnd(&strm);

    // Create return structure
    INFLATED_DATA_PTR result = (INFLATED_DATA_PTR)malloc(sizeof(INFLATED_DATA));
    if (!result) {
        printf("Failed to allocate result structure\n");
        free(out_buffer);
        return NULL;
    }

    // Fill the result
    result->size = actual_size;
    result->data = out_buffer; // Transfer ownership of buffer

    return result;
}

// Fallback function with dynamic buffer allocation
INFLATED_DATA_PTR in_flate_dynamic(unsigned char* compressed_data, unsigned long compressed_size) {
    z_stream strm;
    memset(&strm, 0, sizeof(strm));
    
    // Initialize zlib for the second attempt
    strm.zalloc = Z_NULL;
    strm.zfree = Z_NULL;
    strm.opaque = Z_NULL;
    strm.avail_in = compressed_size;
    strm.next_in = compressed_data;
    
    int ret = inflateInit(&strm);
    if (ret != Z_OK) {
        printf("inflateInit failed in dynamic mode: %s\n", strm.msg ? strm.msg : "unknown error");
        return NULL;
    }

    // Start with a reasonable buffer size
    unsigned long buffer_size = compressed_size * 4;
    unsigned char* buffer = (unsigned char*)malloc(buffer_size);
    if (!buffer) {
        printf("Failed to allocate buffer in dynamic mode\n");
        inflateEnd(&strm);
        return NULL;
    }

    strm.avail_out = buffer_size;
    strm.next_out = buffer;

    // Keep inflating until done or error
    while (1) {
        ret = inflate(&strm, Z_NO_FLUSH);
        
        if (ret == Z_STREAM_END) {
            break; // Decompression complete
        }
        
        if (ret != Z_OK) {
            printf("inflate error in dynamic mode: %d (%s)\n", ret, strm.msg ? strm.msg : "unknown error");
            free(buffer);
            inflateEnd(&strm);
            return NULL;
        }
        
        // If we've used all output space but there's still input left
        if (strm.avail_out == 0) {
            // Double the buffer size
            unsigned long new_size = buffer_size * 2;
            unsigned char* new_buffer = (unsigned char*)realloc(buffer, new_size);
            
            if (!new_buffer) {
                printf("Failed to reallocate buffer to size %lu\n", new_size);
                free(buffer);
                inflateEnd(&strm);
                return NULL;
            }
            
            // Update buffer pointer and zlib state
            buffer = new_buffer;
            strm.next_out = buffer + buffer_size; // Point to the new portion
            strm.avail_out = buffer_size; // The new portion size
            buffer_size = new_size;
        }
    }

    // Get final size and clean up zlib
    unsigned long actual_size = strm.total_out;
    inflateEnd(&strm);
    
    // Shrink buffer to actual size if needed
    if (actual_size < buffer_size) {
        unsigned char* final_buffer = (unsigned char*)realloc(buffer, actual_size);
        if (final_buffer) {
            buffer = final_buffer;
        }
    }

    // Create and return result
    INFLATED_DATA_PTR result = (INFLATED_DATA_PTR)malloc(sizeof(INFLATED_DATA));
    if (!result) {
        printf("Failed to allocate result structure in dynamic mode\n");
        free(buffer);
        return NULL;
    }
    
    result->size = actual_size;
    result->data = buffer;
    
    return result;
}

EXPORT_IMPORT DEFLATED_DATA_PTR de_flate(unsigned char* data, unsigned long data_size) 
{
    /* ZLIB stuff */    
    int ret;
    z_stream strm;
    unsigned char* compressed_pixels;
    unsigned long compressed_pixels_size;
    // allocate deflate state 
    strm.zalloc = Z_NULL;
    strm.zfree = Z_NULL;
    strm.opaque = Z_NULL;
    ret = deflateInit(&strm, Z_DEFAULT_COMPRESSION);
    if (ret != Z_OK)
    {
        //handle error
        printf("Error ret != Z_OK");
    }
    // compress input buffer    
    strm.avail_in = data_size;
    strm.next_in = data;
    // allocate output buffer
    compressed_pixels_size = deflateBound(&strm, data_size);
    compressed_pixels = (unsigned char *)malloc(compressed_pixels_size);
  
    strm.avail_out = compressed_pixels_size;
    strm.next_out = compressed_pixels;
  
    ret = deflate(&strm, Z_FINISH);
    if (ret != Z_STREAM_END) {
        // handle error
  
        printf("Error ret != Z_STREAM_END");
    }
  
    // clean up and return 
    (void)deflateEnd(&strm);   
    /* ZLIB stuff */
  
    DEFLATED_DATA_PTR ptr = malloc(sizeof(DEFLATED_DATA));
  
    ptr->size = compressed_pixels_size;
    ptr->data = compressed_pixels;
  
    return ptr; 
}

/* ****************************************************************************************** */
/* ******************************************* CRC ****************************************** */
/* ****************************************************************************************** */  
/* Table of CRCs of all 8-bit messages. */
unsigned long crc_table[256];   
/* Flag: has the table been computed? Initially false. */
int crc_table_computed = 0;

/* Make the table for a fast CRC. */
void make_crc_table(void)
{
    unsigned long c;
    int n, k;
   
    for (n = 0; n < 256; n++) {

        c = (unsigned long) n;

        //printf(" c =  ");

        for (k = 0; k < 8; k++) {

            if (c & 1) {

                c = 0xedb88320L ^ (c >> 1);

            } else {

                c = c >> 1;
            }        
        }
    
        crc_table[n] = c;
    }

    crc_table_computed = 1;
}

/* Update a running CRC with the bytes buf[0..len-1]--the CRC
   should be initialized to all 1's, and the transmitted value
   is the 1's complement of the final running CRC (see the
   crc() routine below)). */   
EXPORT_IMPORT unsigned long update_crc(unsigned long crc, unsigned char *buf, unsigned int len)
{
    unsigned long c = crc;
    int n;
   
    if (!crc_table_computed) {

       make_crc_table();
    }

    for (n = 0; n < len; n++) {

       c = crc_table[(c ^ buf[n]) & 0xff] ^ (c >> 8);
    }
    return c;
}

/* ****************************************************************************************** */
/* ***************************************** CRC Ends *************************************** */
/* ****************************************************************************************** */


/* ****************************************************************************************** */
/* ***************************************** Endiness *************************************** */
/* ****************************************************************************************** */
EXPORT_IMPORT int little_endian_read_u32(unsigned char* ptr)
{

    return 0;
}

EXPORT_IMPORT unsigned char* little_endian_write_u32(unsigned char* ptr, unsigned int value)
{

    return NULL;
} 

/*
    @ptr has an array of size 4, this array has a big-endian representation of a value.
    This function will turn the big-endian representation to it equivalent value in little-endian
 */
EXPORT_IMPORT unsigned int big_endian_read_u32(unsigned char* ptr)
{
    
    return (ptr[0] << 24 | ptr[1] << 16 | ptr[2] << 8 | ptr[3]);
}

/*
    @ptr has array of size 4, value is stored in this array in its big-endian form/representation
 */
EXPORT_IMPORT void big_endian_write_u32(unsigned char* ptr, unsigned int value)
{  

    ptr[0] = (value & 0xff000000) >> 24;
    ptr[1] = (value & 0x00ff0000) >> 16;
    ptr[2] = (value & 0x0000ff00) >> 8;
    ptr[3] = (value & 0x000000ff);    
}
/* ****************************************************************************************** */
/* **************************************** Endiness Ends *********************************** */
/* ****************************************************************************************** */