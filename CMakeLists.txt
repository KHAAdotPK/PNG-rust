cmake_minimum_required(VERSION 3.24)
project(sundry)

# Add include directories
include_directories("C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.37.32822\\include")

# ------------
#  sundry.dll
# ------------
# Add the source file
set(SOURCES_SUNDRY ${CMAKE_SOURCE_DIR}/rust/C/sundry.c)
# Set the target to build a shared library
add_library(sundry SHARED ${SOURCES_SUNDRY})

# https://stackoverflow.com/questions/34799916/copy-file-from-source-directory-to-binary-directory-using-cmake
# Add a custom command to copy clap.dll to the desired folder
add_custom_command(TARGET sundry POST_BUILD    
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:sundry> C:/metaverse-tools/editor/$<TARGET_FILE_NAME:sundry>
    DEPENDS sundry
)



