use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.as_ref().join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(entry.path(), dest_path)?;
        } else {
            // Only update if modified to not kill compilation time
            if !dest_path.exists() || 
                (entry.metadata()?.modified()? > dest_path.metadata()?.modified()?) {
                fs::copy(entry.path(), dest_path)?;
            }
        }
    }
    Ok(())
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not found");
    
    // out_dir is something like target/debug/build/zombiegame-hash/out
    // We want to target target/debug
    let mut target_dir = PathBuf::from(&out_dir);
    target_dir.pop();
    target_dir.pop();
    target_dir.pop();
    
    let src = Path::new("assets");
    let dest = target_dir.join("assets");
    
    if src.exists() {
        if let Err(e) = copy_dir_all(src, dest) {
            println!("cargo:warning=Failed to copy assets: {}", e);
        }
    }
    
    println!("cargo:rerun-if-changed=assets");
}
