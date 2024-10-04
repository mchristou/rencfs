#[cfg(unix)]
use atomic_write_file::unix::OpenOptionsExt;
use atomic_write_file::AtomicWriteFile;
use futures_util::TryStreamExt;
use std::path::Path;
use std::{fs, io};
use tokio_stream::wrappers::ReadDirStream;

/// Recursively moves the content of a directory to another.
/// It will create destination directory if it doesn't exist. It will delete the source directory after the move.
pub async fn rename_dir_content(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "source directory does not exist",
        ));
    }

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    let read_dir = tokio::fs::read_dir(src).await?;
    let read_dir_stream = ReadDirStream::new(read_dir);
    let vec = read_dir_stream.try_collect::<Vec<_>>().await?;
    let entries = vec.iter().collect::<Vec<_>>();
    for entry in entries {
        let dst = dst.join(entry.file_name());
        if entry.path().is_dir() {
            fs::create_dir_all(&dst)?;
            Box::pin(rename_dir_content(&entry.path(), &dst)).await?;
        } else {
            fs::rename(entry.path(), dst)?;
        }
    }

    fs::remove_dir(src)?;

    Ok(())
}

pub fn open_atomic_write(file: &Path) -> io::Result<AtomicWriteFile> {
    let mut opt = AtomicWriteFile::options();
    opt.read(true);
    #[cfg(unix)]
    opt.preserve_mode(true).preserve_owner(true);
    opt.open(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_rename_dir_content_success() {
        let src_dir = tempdir().unwrap();
        let dst_dir = tempdir().unwrap();

        let src_path = src_dir.path().join("source_dir");
        fs::create_dir(&src_path).unwrap();
        fs::write(src_path.join("file1.txt"), "This is file 1.").unwrap();
        fs::create_dir(src_path.join("subdir1")).unwrap();
        fs::write(src_path.join("subdir1/file2.txt"), "This is file 2.").unwrap();

        let result = rename_dir_content(&src_path, dst_dir.path()).await;

        assert!(result.is_ok());

        let moved_file = dst_dir.path().join("file1.txt");
        let moved_subdir = dst_dir.path().join("subdir1/file2.txt");
        assert!(moved_file.exists());
        assert!(moved_subdir.exists());
        assert!(!src_path.exists()); // Ensure the source directory is deleted
    }

    #[tokio::test]
    async fn test_rename_dir_content_src_not_found() {
        let dst_dir = tempdir().unwrap();
        let non_existent_src = PathBuf::from("/non_existent_dir");

        let result = rename_dir_content(&non_existent_src, dst_dir.path()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn test_rename_dir_content_dst_not_exist() {
        let src_dir = tempdir().unwrap();
        let src_path = src_dir.path().join("source_dir");
        fs::create_dir(&src_path).unwrap();

        fs::write(src_path.join("file1.txt"), "This is file 1.").unwrap();

        let dst_path = PathBuf::from("/tmp/non_existent_dir/dst");

        let result = rename_dir_content(&src_path, &dst_path).await;

        println!("{result:?}");
        assert!(result.is_ok());

        assert!(dst_path.exists());
        assert!(dst_path.join("file1.txt").exists());

        fs::remove_dir_all(dst_path).unwrap();
    }
}
