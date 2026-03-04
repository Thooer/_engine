use std::fs;
use std::io;
use std::path::Path;

use super::{FileSystem, LocalFileSystem};

impl FileSystem for LocalFileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let full = if let Some(root) = &self.root {
            root.join(path)
        } else {
            path.to_path_buf()
        };
        fs::read_to_string(full)
    }

    fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>> {
        let full = if let Some(root) = &self.root {
            root.join(path)
        } else {
            path.to_path_buf()
        };
        fs::read(full)
    }
}

