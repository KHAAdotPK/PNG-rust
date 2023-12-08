/*
    lib/rust/C/sundry.c
    Q@khaa.pk
 */

#include "sundry.h"

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