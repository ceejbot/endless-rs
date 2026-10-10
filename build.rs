//! Makes the linker script available to patch crates.
//!
//! Copies `link/patch_imx.ld` into this crate's `OUT_DIR` and adds that
//! directory to the linker search path. Cargo passes `-L` flags from a
//! dependency's build script through to the final link, so a patch crate
//! only needs `-C link-arg=-Tpatch_imx.ld` in its `.cargo/config.toml` and
//! never has to know where `endless-rs` lives on disk.

use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let script = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("link")
        .join("patch_imx.ld");
    fs::copy(&script, out_dir.join("patch_imx.ld"))?;
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed={}", script.display());
    Ok(())
}
