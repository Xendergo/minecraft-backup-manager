use super::backup_writer::write_files_with_wd;
use crate::backup::backup::backup_reader::BackupReader;
use crate::backup::backup::backup_writer::BackupWriter;
use crate::backup::BackupArgs;
use crate::try_option;
use crate::utils::BackupsFolder;
use anyhow::anyhow;
use anyhow::Result;
use quartz_nbt::io::Flavor;
use quartz_nbt::serde::{deserialize, serialize};
use serde::Deserialize;
use serde::Serialize;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

pub const PREV_BACKUP_PREFIX: &'static str = "__in_prev_backup_";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupData {
    pub previous: Option<PathBuf>,
    pub current: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Backup {
    data: BackupData,
}

impl Backup {
    pub(super) fn new(data: BackupData) -> Backup {
        Backup { data: data }
    }

    pub fn get(path: impl AsRef<Path>) -> Result<Option<Backup>> {
        let mut reader = try_option!(BackupReader::new(&path));

        let mut file = reader.get_file("archive_data.nbt").ok_or(anyhow!("The file archive_data.nbt is missing from the backup `{}`, this file contains which backup comes before it, which is important for incremental backups", path.as_ref().file_name().unwrap().to_string_lossy()))?;

        let mut data_buf = Vec::new();
        file.read_to_end(&mut data_buf)?;

        let archive_data = deserialize::<BackupData>(&data_buf, Flavor::Uncompressed)?;

        Ok(Some(Backup {
            data: archive_data.0,
        }))
    }

    pub fn create(from: &PathBuf, backups_dir: BackupsFolder, args: &BackupArgs) -> Result<Backup> {
        let prev = backups_dir.current_backup()?;

        let data = BackupData {
            previous: prev,
            current: backups_dir.dir().join(&args.name),
        };

        let mut backup_writer = BackupWriter::new(&from, data.clone(), &args)?;

        backups_dir.set_current_backup(&args.name)?;

        write_files_with_wd(&mut backup_writer, from)?;

        backup_writer.add_new_file(
            &from.join("archive_data.nbt"),
            (&mut &serialize(&data, Some(""), Flavor::Uncompressed)?[..]) as &mut dyn Read,
        )?;

        Ok(Backup::new(data))
    }

    pub fn get_reader(&self) -> Result<BackupReader> {
        BackupReader::new(&self.get_data().current)?
            .ok_or(anyhow!("The backup `{}` doesn't exist", self.get_name()))
    }

    pub fn get_reader_with_file(
        &self,
        file_path: impl AsRef<Path>,
    ) -> Result<Option<BackupReader>> {
        try_option!(self.prev()).get_reader_with_file_including_current(file_path)
    }

    fn get_reader_with_file_including_current(
        &self,
        file_path: impl AsRef<Path>,
    ) -> Result<Option<BackupReader>> {
        let reader = self.get_reader()?;
        let path_string = file_path.as_ref().as_os_str().to_str().ok_or(anyhow!(
            "The path to one of the files isn't valid unicode: {}",
            file_path.as_ref().as_os_str().to_string_lossy()
        ))?;

        let indication_string = &(PREV_BACKUP_PREFIX.to_owned() + path_string);

        if reader.file_names().any(|v| v == indication_string) {
            let prev = self.prev()?.ok_or(anyhow!("The backup `{}` doesn't indicate a previous backup, but includes a file referencing a previous backup, `{}`", self.get_name(), path_string))?;
            return prev.get_reader_with_file_including_current(file_path);
        } else if reader.file_names().any(|v| v == path_string) {
            return Ok(Some(reader));
        }

        Ok(None)
    }

    pub fn get_data(&self) -> &BackupData {
        &self.data
    }

    pub fn prev(&self) -> Result<Option<Backup>> {
        Backup::get(try_option!(no_try, &self.data.previous))
    }

    pub fn get_name(&self) -> String {
        self.get_data()
            .current
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
