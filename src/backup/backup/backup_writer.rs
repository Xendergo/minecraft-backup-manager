use anyhow::Error;
use anyhow::Result;
use ring::digest::{digest, SHA256};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::backup::BackupArgs;
use crate::backup::BackupType;
use crate::backup::BackupType::*;
use crate::utils::option_open;

use super::Backup;
use super::BackupData;
use super::PREV_BACKUP_PREFIX;

pub struct BackupWriter {
    backup: ZipWriter<File>,
    source_dir: PathBuf,
    backup_type: BackupType,
    data: BackupData,
}

impl BackupWriter {
    pub fn new(
        source_dir: &dyn AsRef<Path>,
        backups_data: BackupData,
        args: &BackupArgs,
    ) -> Result<BackupWriter> {
        Ok(BackupWriter {
            source_dir: source_dir.as_ref().to_path_buf(),
            backup: ZipWriter::new(File::create(&backups_data.current)?),
            backup_type: args.backup_type,
            data: backups_data,
        })
    }

    fn out_dir(&self, source: &dyn AsRef<Path>) -> Result<PathBuf> {
        Ok(source
            .as_ref()
            .to_path_buf()
            .strip_prefix(&self.source_dir)?
            .to_path_buf())
    }

    pub fn add_file(&mut self, source: &dyn AsRef<Path>) -> Result<()> {
        let mut data = Vec::new();
        option_open(source)?
            .expect("Tried to add a file that doesn't exist to the backup")
            .read_to_end(&mut data)?;

        let out_dir = self.out_dir(source)?;

        match self.backup_type {
            Partial => {
                let _digest = digest(&SHA256, &data);
                let hash = _digest.as_ref();

                match Backup::new(BackupData::clone(&self.data)).get_reader_with_file(&out_dir)? {
                    Some(mut v) => {
                        let mut prev_file = v.get_file(out_dir.to_str().unwrap()).unwrap();

                        let mut prev_data = Vec::new();
                        prev_file.read_to_end(&mut prev_data)?;

                        let _digest = digest(&SHA256, &prev_data);
                        let prev_hash = _digest.as_ref();

                        if hash == prev_hash {
                            self.mark_in_prev(source)?;
                        } else {
                            self.write_data(&mut data, source)?;
                        }
                    }
                    None => {
                        self.write_data(&mut data, source)?;
                    }
                }
            }
            Full => {
                self.write_data(&mut data, source)?;
            }
        }

        Ok(())
    }

    fn write_data(&mut self, data: &mut [u8], source: &dyn AsRef<Path>) -> Result<()> {
        let dir = self.out_dir(source)?;

        self.backup
            .start_file(dir.to_str().unwrap(), FileOptions::default())?;
        self.backup.write_all(data)?;

        Ok(())
    }

    fn mark_in_prev(&mut self, source: &dyn AsRef<Path>) -> Result<()> {
        let dir = self.out_dir(source)?;

        self.write_data(
            &mut [],
            &source
                .as_ref()
                .parent()
                .unwrap()
                .join(PREV_BACKUP_PREFIX.to_owned() + dir.file_name().unwrap().to_str().unwrap()),
        )
    }

    pub fn add_directory(&mut self, source: &dyn AsRef<Path>) -> Result<()> {
        let dir = self.out_dir(source)?;

        self.backup
            .add_directory(dir.to_str().unwrap(), FileOptions::default())?;

        Ok(())
    }

    pub fn add_new_file(&mut self, source: &dyn AsRef<Path>, data: &mut dyn Read) -> Result<()> {
        let mut data_buf = Vec::new();
        data.read_to_end(&mut data_buf)?;

        self.write_data(&mut data_buf, source)?;

        Ok(())
    }
}

pub fn write_files_with_wd(writer: &mut BackupWriter, from_trait: &dyn AsRef<Path>) -> Result<()> {
    let from = from_trait.as_ref().to_path_buf();

    if from.is_dir() {
        writer.add_directory(&from)?;

        for item in fs::read_dir(&from)? {
            let path = item?.file_name();
            let path_str = path.to_str().unwrap();

            if &path_str.chars().take(1).collect::<String>() == "." {
                continue;
            }

            if &path_str.chars().take(7).collect::<String>() == PREV_BACKUP_PREFIX {
                return Err(Error::msg(format!(
                    "File names may not start with {} (this'd break incremental backups): {}",
                    PREV_BACKUP_PREFIX,
                    from.join(path).to_str().unwrap()
                )));
            }

            write_files_with_wd(writer, &from.join(path))?;
        }
    } else if from.is_file() {
        writer.add_file(&from)?;
    } else {
        panic!(
            "Found a path that is neither a directory nor a file: {:?}",
            from
        )
    }

    Ok(())
}
