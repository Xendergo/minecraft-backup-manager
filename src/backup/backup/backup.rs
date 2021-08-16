use super::backup_writer::write_files_with_wd;
use crate::utils::BackupsFolder;
use crate::utils::WriteNewFile;
use anyhow::Error;
use anyhow::Result;
use flate2::read::GzDecoder;
use quartz_nbt::io::Flavor;
use quartz_nbt::serde::deserialize;
use quartz_nbt::serde::serialize;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;
use tar::Builder;

#[derive(Serialize, Deserialize, Debug)]
struct BackupData {
    previous: Option<PathBuf>,
    current: PathBuf,
}

pub struct Backup {
    data: BackupData,
}

impl Backup {
    fn new(data: BackupData) -> Backup {
        Backup { data: data }
    }

    pub fn create(
        from: &PathBuf,
        archive: &mut Builder<impl Write>,
        backups_dir: BackupsFolder,
        name: String,
    ) -> Result<Backup> {
        let prev = backups_dir.current_backup()?;

        let data = BackupData {
            previous: prev,
            current: backups_dir.dir().join(&name),
        };

        archive.append_new_file(
            "archive_data.nbt",
            &serialize(&data, Some(""), Flavor::Uncompressed)?[..],
        )?;

        backups_dir.set_current_backup(name)?;

        write_files_with_wd(from, archive, &PathBuf::from(""))?;

        Ok(Backup::new(data))
    }

    pub fn get(path: PathBuf) -> Result<Backup> {
        let mut out = Vec::new();

        Archive::new(GzDecoder::new(File::open(path)?))
            .entries()?
            .nth(0)
            .ok_or(Error::msg("The backup being opened is empty"))??
            .read_to_end(&mut out)?;

        let v = deserialize::<BackupData>(&out, Flavor::Uncompressed)?;

        Ok(Backup::new(v.0))
    }

    pub fn get_reader(&self) -> Result<Archive<GzDecoder<File>>> {
        Ok(Archive::new(GzDecoder::new(File::open(
            self.data.current.clone(),
        )?)))
    }

    // pub fn backup_iterator(&self) -> BackupIterator {}
}
