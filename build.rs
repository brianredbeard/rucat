// build.rs - Optional build script for conditional compilation

fn main() {
    // Detect platform capabilities at build time

    #[cfg(target_family = "unix")]
    {
        // Check if we have the necessary headers for extended attributes
        println!("cargo:rerun-if-changed=build.rs");
    }

    #[cfg(target_family = "windows")]
    {
        // Windows-specific build configuration
        println!("cargo:rerun-if-changed=build.rs");
    }
}
