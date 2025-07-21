cmake_minimum_required(VERSION 3.10)
project(tui)

# Build C library first
add_subdirectory(c_src)

# Rust build will be handled by Cargo via build.rs
