use anyhow::Error;
use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::env::current_dir;
use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::io;
use std::path::Path;
use std::path::PathBuf;

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

pub struct TempFolder {
    dir: PathBuf,
}

impl TempFolder {
    pub fn new() -> io::Result<TempFolder> {
        use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
        use std::env;
        let random: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        let temp_dir = env::temp_dir().join(random);
        fs::create_dir(&temp_dir)?;

        Ok(TempFolder { dir: temp_dir })
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        // it's a tmp folder, so if it can't be cleaned up now, it'll be cleaned up later
        let _ = fs::remove_dir_all(&self.dir);
    }
}

impl AsRef<Path> for TempFolder {
    fn as_ref(&self) -> &Path {
        &self.dir
    }
}

pub fn copy_and_hash(from: &PathBuf, to: &PathBuf) -> Result<u64> {
    copy_and_hash_with_wd(from, to, &PathBuf::from(""))
}

fn copy_and_hash_with_wd(from_root: &PathBuf, to_root: &PathBuf, cwd: &PathBuf) -> Result<u64> {
    let from = &from_root.join(cwd);
    let to = &to_root.join(cwd);

    let mut hasher = DefaultHasher::new();

    to.hash(&mut hasher);

    if from.is_dir() {
        if cwd != &PathBuf::from("") {
            fs::create_dir(to)?;
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

            let hash = copy_and_hash_with_wd(from_root, to_root, &cwd.join(path))?;

            hasher.write_u64(hash);
        }
    } else if from.is_file() {
        fs::copy(from, to)?;

        hasher.write(&fs::read(from)?)
    } else {
        panic!(
            "Found a path that is neither a directory nor a file: {:?}",
            from
        )
    }

    let hash = hasher.finish();

    fs::write(
        to.parent().unwrap().join(format!(
            "__hash_{}",
            to.file_name().unwrap().to_str().unwrap()
        )),
        hash.to_le_bytes(),
    )?;

    Ok(hash)
}
