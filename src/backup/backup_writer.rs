use crate::utils::BackupsFolder;
use crate::utils::TempFile;
use crate::utils::WriteNewFile;
use anyhow::{Error, Result};
use flate2::read::GzDecoder;
use quartz_nbt::io::Flavor;
use quartz_nbt::serde::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::{Read, Write};
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

fn write_files_with_wd(
    from_root: &PathBuf,
    archive: &mut Builder<impl Write>,
    cwd: &PathBuf,
) -> Result<u64> {
    let from = &from_root.join(cwd);

    let mut hasher = DefaultHasher::new();

    cwd.hash(&mut hasher);

    if from.is_dir() {
        // if cwd != &PathBuf::from("") {
        //     archive.append_dir(cwd, from)?;
        // }

        for item in fs::read_dir(from)? {
            let path = item?.file_name();
            let path_str = path.to_str().unwrap();

            if &path_str.chars().take(1).collect::<String>() == "." {
                continue;
            }

            if &path_str.chars().take(7).collect::<String>() == "__hash_" {
                return Err(Error::msg(format!(
                    "File names may not start with __hash_ (this'd break change detection): {}",
                    cwd.join(path).to_str().unwrap()
                )));
            }

            let hash = write_files_with_wd(from_root, archive, &cwd.join(path))?;

            hasher.write_u64(hash);
        }
    } else if from.is_file() {
        archive.append_file(cwd, &mut File::open(from)?)?;

        hasher.write(&fs::read(from)?)
    } else {
        panic!(
            "Found a path that is neither a directory nor a file: {:?}",
            from
        )
    }

    let hash = hasher.finish();

    let mut tmp_file = TempFile::new()?;
    tmp_file.write(&hash.to_le_bytes())?;

    if let Some(parent) = cwd.parent() {
        archive.append_file(
            parent.join(format!(
                "__hash_{}",
                cwd.file_name().unwrap().to_str().unwrap()
            )),
            &mut File::open(tmp_file)?,
        )?;
    }

    Ok(hash)
}

// pub struct BackupIterator {
//     current_backup_data: BackupData,
// }

// impl BackupIterator {
//     fn new(backup: BackupData) -> BackupIterator {
//         BackupIterator {
//             current_backup_data: backup,
//         }
//     }
// }

// impl Iterator for BackupIterator {
//     type Item = Result<Backup>;
// }
