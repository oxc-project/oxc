use std::{
    io,
    path::{Path, PathBuf},
};

use crate::{FileMetadata, FileSystem};

pub struct MemoryFS {
    fs: vfs::MemoryFS,
}

impl Default for MemoryFS {
    fn default() -> Self {
        let fs = vfs::MemoryFS::new();
        Self { fs }
    }
}

impl MemoryFS {
    /// # Panics
    ///
    /// * Fails to create directory
    /// * Fails to write file
    pub fn new(data: &[(&'static str, &'static str)]) -> Self {
        use vfs::FileSystem;
        let fs = vfs::MemoryFS::default();
        for (path, string) in data {
            // Create all parent directories
            for path in Path::new(path).ancestors().collect::<Vec<_>>().iter().rev() {
                let path = path.to_string_lossy();
                if !fs.exists(path.as_ref()).unwrap() {
                    fs.create_dir(path.as_ref()).unwrap();
                }
            }
            let mut file = fs.create_file(path).unwrap();
            file.write_all(string.as_bytes()).unwrap();
        }
        Self { fs }
    }
}

impl FileSystem for MemoryFS {
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        use vfs::FileSystem;
        let mut file = self
            .fs
            .open_file(path.as_ref().to_string_lossy().as_ref())
            .map_err(|err| io::Error::new(io::ErrorKind::NotFound, err))?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        Ok(buffer)
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata> {
        use vfs::FileSystem;
        let metadata = self
            .fs
            .metadata(path.as_ref().to_string_lossy().as_ref())
            .map_err(|err| io::Error::new(io::ErrorKind::NotFound, err))?;
        let is_file = metadata.file_type == vfs::VfsFileType::File;
        let is_dir = metadata.file_type == vfs::VfsFileType::Directory;
        Ok(FileMetadata::new(is_file, is_dir))
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        Ok(path.as_ref().to_path_buf())
    }
}
