/*
    lib/rust/C/sundry.h
    Q@khaa.pk
 */

#include <stdio.h>

#include "clap.h"

#ifndef SUNDRY_HH
#define SUNDRY_HH

#ifdef __cplusplus
extern "C" {  // only need to export C interface if
              // used by C++ source code      
#endif

EXPORT_IMPORT int little_endian_read_u32(unsigned char* ptr);
EXPORT_IMPORT unsigned char* little_endian_write_u32(unsigned char* ptr, unsigned int value);
EXPORT_IMPORT unsigned int big_endian_read_u32(unsigned char* ptr);
EXPORT_IMPORT void big_endian_write_u32(unsigned char* ptr, unsigned int value);

#ifdef __cplusplus
}
#endif

#endif
