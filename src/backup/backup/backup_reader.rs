use anyhow::Result;
use std::fs::File;
use std::path::PathBuf;
use zip::read::ZipFile;
use zip::ZipArchive;

pub struct BackupReader {
    backup: ZipArchive<File>,
}

impl BackupReader {
    pub fn new(path: PathBuf) -> Result<BackupReader> {
        Ok(BackupReader {
            backup: ZipArchive::new(File::open(path)?)?,
        })
    }

    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.backup.file_names()
    }

    pub fn get_file(&mut self, name: &str) -> Result<ZipFile> {
        Ok(self.backup.by_name(name)?)
    }
}
