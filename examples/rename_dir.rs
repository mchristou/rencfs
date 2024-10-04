use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

use rencfs::fs_util::rename_dir_content;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let mut args = std::env::args();
    let _inner = args.next();

    let src = args.next().unwrap_or("/tmp/source_dir".to_string());
    let dst = args.next().unwrap_or("/tmp/destination_dir".to_string());

    let src = Path::new(&src);
    if !src.exists() {
        setup_source_directory(src).unwrap();
    }

    let dst = Path::new(&dst);

    if dst.exists() {
        warn!("Destination folder already exists, will modify it");
    }

    info!("Source directory before renaming:");
    print_directory_contents(src, 0).unwrap();

    rename_dir_content(src, dst).await.unwrap();

    info!("\nDestination directory after renaming:");
    print_directory_contents(dst, 0).unwrap();

    Ok(())
}

fn setup_source_directory(src: &Path) -> Result<()> {
    info!("Creating source directory");
    fs::create_dir_all(src.join("subdir1")).unwrap();
    fs::create_dir_all(src.join("subdir2")).unwrap();

    fs::write(src.join("file1.txt"), "This is file 1.").unwrap();
    fs::write(src.join("file2.txt"), "This is file 2.").unwrap();
    fs::write(
        src.join("subdir1").join("file3.txt"),
        "This is file 3 in subdir1.",
    )
    .unwrap();
    fs::write(
        src.join("subdir2").join("file4.txt"),
        "This is file 4 in subdir2.",
    )
    .unwrap();

    Ok(())
}

fn print_directory_contents(dir: &Path, depth: usize) -> Result<()> {
    if dir.is_dir() {
        let indent = "  ".repeat(depth);
        println!("{}{}", indent, dir.display());

        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                print_directory_contents(&path, depth + 1).unwrap();
            } else {
                println!("{}{:?}", indent, path.file_name().unwrap());
            }
        }
    }
    Ok(())
}
