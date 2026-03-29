fn main() {
    // Point the linker at the endless-rs linker script directory.
    let ld_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("link");
    println!("cargo:rustc-link-search={}", ld_dir.display());
    println!("cargo:rerun-if-changed=../../link/patch_imx.ld");
}
