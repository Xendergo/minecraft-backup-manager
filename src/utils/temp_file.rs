use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub struct TempFile {
    dir: PathBuf,
}

impl TempFile {
    pub fn new() -> TempFile {
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
                OpenOptions::new()
                    .read(true)
                    .create_new(true)
                    .truncate(true)
                    .open(&temp_file)
                    .expect("Couldn't create the temporary file");
                break temp_file;
            }
        };

        TempFile { dir: temp_file }
    }

    pub fn get_readonly(&self) -> io::Result<File> {
        File::open(&self.dir)
    }

    pub fn get_writeonly(&mut self) -> io::Result<File> {
        OpenOptions::new().write(true).open(&self.dir)
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
