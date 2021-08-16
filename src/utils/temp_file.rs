use core::ops::Deref;
use core::ops::DerefMut;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;

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

impl AsRef<Path> for TempFile {
    fn as_ref(&self) -> &Path {
        &self.dir
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
