# PNG-rust
A Rust crate to create and edit PNG files. 

## Build Instructions

Follow these steps to build `PNG-rust`:

1. **CMake Build:** Execute CMake on the `CMakeLists.txt` file to build the required libraries (`sundry.lib`, `sundry.dll`). 

    ```bash
    cmake ./CMakeLists.txt
    ```

2. **Copy Libraries:** Copy the newly built files (`sundry.lib`, `sundry.dll`) into the root directory (`./`).

3. **Cargo Build:** Finally, execute `cargo build` to build your crate which uses `PNG-rust`. 

    ```bash
    cargo build
    ```
Please note that the build process is still a work in progress and will be continuously improved for a more seamless experience.

Feel free to contribute and report issues.  

### License
This project is governed by a license, the details of which can be located in the accompanying file named 'LICENSE.' Please refer to this file for comprehensive information.
