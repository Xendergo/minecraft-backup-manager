use anyhow::Result;
use std::fs::File;
use std::path::Path;
use zip::read::ZipFile;
use zip::ZipArchive;

use crate::try_option;
use crate::utils::option_open;

pub struct BackupReader {
    backup: ZipArchive<File>,
}

impl BackupReader {
    pub fn new(path: impl AsRef<Path>) -> Result<Option<BackupReader>> {
        let file = try_option!(option_open(path));

        Ok(Some(BackupReader {
            backup: ZipArchive::new(file)?,
        }))
    }

    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.backup.file_names()
    }

    pub fn get_file(&mut self, name: &str) -> Option<ZipFile> {
        match self.backup.by_name(name) {
            Ok(v) => Some(v),
            Err(e) => {
                println!("{:?}", e);
                None
            }
        }
    }
}
