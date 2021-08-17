use super::backup_writer::write_files_with_wd;
use crate::backup::backup::backup_writer::BackupWriter;
use crate::backup::BackupArgs;
use crate::utils::BackupsFolder;
use anyhow::Result;
use quartz_nbt::io::Flavor;
use quartz_nbt::serde::serialize;
use serde::Deserialize;
use serde::Serialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupData {
    pub previous: Option<PathBuf>,
    pub current: PathBuf,
}

pub struct Backup {
    data: BackupData,
}

impl Backup {
    fn new(data: BackupData) -> Backup {
        Backup { data: data }
    }

    pub fn create(from: &PathBuf, backups_dir: BackupsFolder, args: &BackupArgs) -> Result<Backup> {
        let prev = backups_dir.current_backup()?;

        let data = BackupData {
            previous: prev,
            current: backups_dir.dir().join(&args.name),
        };

        let mut backup_writer = BackupWriter::new(&from, &backups_dir, &args.name)?;

        backups_dir.set_current_backup(&args.name)?;

        write_files_with_wd(&mut backup_writer, from)?;

        backup_writer.add_new_file(
            &from.join("archive_data.nbt"),
            (&mut &serialize(&data, Some(""), Flavor::Uncompressed)?[..]) as &mut dyn Read,
        )?;

        Ok(Backup::new(data))
    }

    // pub fn get(path: PathBuf) -> Result<Backup> {
    //     let mut out = Vec::new();

    //     Archive::new(GzDecoder::new(File::open(path)?))
    //         .entries()?
    //         .nth(0)
    //         .ok_or(Error::msg("The backup being opened is empty"))??
    //         .read_to_end(&mut out)?;

    //     let v = deserialize::<BackupData>(&out, Flavor::Uncompressed)?;

    //     Ok(Backup::new(v.0))
    // }

    // pub fn get_reader(&self) -> Result<Archive<GzDecoder<File>>> {
    //     Ok(Archive::new(GzDecoder::new(File::open(
    //         self.data.current.clone(),
    //     )?)))
    // }

    pub fn get_data(&self) -> &BackupData {
        &self.data
    }

    // pub fn backup_iterator(&self) -> BackupIterator {}
}
