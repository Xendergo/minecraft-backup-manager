use crate::backup::Backup;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use tar::Archive;

pub struct BackupReader {
    paths: Vec<PathBuf>,
    archive: Archive<GzDecoder<File>>,
}

impl BackupReader {
    pub fn create(backup: Backup) -> io::Result<BackupReader> {
        let file = File::open(backup.get_data().current.clone())?;

        let mut archive = Archive::new(GzDecoder::new(&file));

        let mut paths = Vec::new();

        for item in archive.entries()? {
            paths.push(item?.path()?.into_owned())
        }

        Ok(BackupReader {
            paths: paths,
            archive: Archive::new(GzDecoder::new(file)),
        })
    }
}
