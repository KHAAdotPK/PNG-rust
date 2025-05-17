/*
    lib/rust/C/sundry.h
    Q@khaa.pk
 */

#include "clap.h"

#ifndef SUNDRY_HH
#define SUNDRY_HH

typedef struct deflated_data
{
    unsigned long size;
    unsigned char* data;

}DEFLATED_DATA, INFLATED_DATA;

typedef struct deflated_data* DEFLATED_DATA_PTR;
typedef struct deflated_data* INFLATED_DATA_PTR;

#ifdef __cplusplus
extern "C" {  // only need to export C interface if
              // used by C++ source code      
#endif

EXPORT_IMPORT int little_endian_read_u32(unsigned char* ptr);
EXPORT_IMPORT unsigned char* little_endian_write_u32(unsigned char* ptr, unsigned int value);
EXPORT_IMPORT unsigned int big_endian_read_u32(unsigned char* ptr);
EXPORT_IMPORT void big_endian_write_u32(unsigned char* ptr, unsigned int value);

EXPORT_IMPORT DEFLATED_DATA_PTR de_flate(unsigned char* data, unsigned long data_size);
EXPORT_IMPORT INFLATED_DATA_PTR in_flate(unsigned char* compressed_data, unsigned long compressed_size);

#ifdef __cplusplus
}
#endif

#endif
