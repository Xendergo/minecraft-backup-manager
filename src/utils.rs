use anyhow::Error;
use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::hash::Hasher;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
use std::path::PathBuf;
use tar::Builder;

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
}

impl AsRef<Path> for BackupsFolder {
    fn as_ref(&self) -> &Path {
        &self.dir
    }
}

pub struct TempFile {
    file: File,
    dir: PathBuf,
}

impl TempFile {
    pub fn new() -> io::Result<TempFile> {
        use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
        use std::env;

        let temp_file = loop {
            let random: String = OsRng
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();

            let temp_file = env::temp_dir().join(format!("__minecraft_backup_tmp_file_{}", random));

            if !temp_file.exists() {
                break temp_file;
            }
        };

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .truncate(true)
            .open(&temp_file)?;

        Ok(TempFile {
            dir: temp_file,
            file: file,
        })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // it's a tmp file, so if it can't be cleaned up now, it'll be cleaned up later
        let _ = fs::remove_file(&self.dir);
    }
}

impl Deref for TempFile {
    type Target = File;

    fn deref(&self) -> &File {
        &self.file
    }
}

impl DerefMut for TempFile {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

pub fn copy_and_hash(from: &PathBuf, archive: &mut Builder<impl Write>) -> Result<u64> {
    copy_and_hash_with_wd(from, archive, &PathBuf::from(""))
}

fn copy_and_hash_with_wd(
    from_root: &PathBuf,
    archive: &mut Builder<impl Write>,
    cwd: &PathBuf,
) -> Result<u64> {
    let from = &from_root.join(cwd);

    let mut hasher = DefaultHasher::new();

    cwd.hash(&mut hasher);

    if from.is_dir() {
        if cwd != &PathBuf::from("") {
            archive.append_dir(cwd, from)?;
        }

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

            let hash = copy_and_hash_with_wd(from_root, archive, &cwd.join(path))?;

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
            &mut tmp_file,
        )?;
    }

    Ok(hash)
}
