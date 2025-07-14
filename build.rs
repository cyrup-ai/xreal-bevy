fn main() {
    #[cfg(target_os = "macos")]
    {
        // Link the objc exception handling for scap compatibility
        println!("cargo:rustc-link-lib=objc");
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}