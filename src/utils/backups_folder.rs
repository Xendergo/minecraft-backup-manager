use anyhow::Error;
use anyhow::Result;
use core::ops::Deref;
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use super::option_open;

pub struct BackupsFolder {
    dir: PathBuf,
}

impl BackupsFolder {
    pub fn get() -> io::Result<BackupsFolder> {
        let cwd = current_dir()?;

        let backups_folder = cwd.join(".backups");

        if !backups_folder.is_dir() {
            fs::create_dir(&backups_folder)?
        }

        Ok(BackupsFolder {
            dir: backups_folder,
        })
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn current_backup(&self) -> Result<Option<PathBuf>> {
        let current_backup_file = self.dir.join(".current");

        if !current_backup_file.exists() {
            File::create(&current_backup_file)?;
        }

        if !current_backup_file.is_file() {
            return Err(Error::msg("Can't get the most recent backup, `.current` is a folder. Please rename the folder"));
        }

        let mut current_backup: String = String::default();
        option_open(current_backup_file)?.unwrap().read_to_string(&mut current_backup)?;

        Ok(if self.dir.join(&current_backup).is_file() {
            Some(self.dir.join(&current_backup))
        } else {
            None
        })
    }

    pub fn set_current_backup(&self, name: &str) -> Result<()> {
        let current_backup_file = self.dir.join(".current");

        let mut file = File::create(current_backup_file)?;

        file.write(name.as_bytes())?;

        Ok(())
    }

    pub fn all_backups(&self) -> io::Result<Box<dyn Iterator<Item = DirEntry>>> {
        Ok(Box::new(
            fs::read_dir(self.dir())?
                .filter(|file| match file {
                    Ok(v) => v.path().extension() == Some(OsStr::new(".zip")),
                    Err(_) => false,
                })
                .map(|v| v.unwrap()),
        ))
    }
}

impl AsRef<Path> for BackupsFolder {
    fn as_ref(&self) -> &Path {
        &self.dir
    }
}

impl Deref for BackupsFolder {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.dir
    }
}
