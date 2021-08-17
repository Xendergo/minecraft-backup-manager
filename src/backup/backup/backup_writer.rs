use anyhow::Error;
use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

pub struct BackupWriter {
    backup_dir: PathBuf,
    source_dir: PathBuf,
}

impl BackupWriter {
    pub fn new(
        source_dir: &dyn AsRef<Path>,
        backups_dir: &dyn AsRef<Path>,
        name: &str,
    ) -> BackupWriter {
        BackupWriter {
            source_dir: source_dir.as_ref().to_path_buf(),
            backup_dir: backups_dir.as_ref().to_path_buf().join(name),
        }
    }

    fn create_writer(&self, source: &dyn AsRef<Path>) -> Result<Box<dyn Write>> {
        let mut dir = self.out_dir(source)?;
        let mut new_name = dir.file_name().unwrap().to_string_lossy().into_owned();

        new_name.push_str(".gz");

        dir.set_file_name(new_name);
        Ok(Box::new(GzEncoder::new(
            File::create(dir)?,
            Compression::fast(),
        )))
    }

    fn out_dir(&self, source: &dyn AsRef<Path>) -> Result<PathBuf> {
        Ok(self.backup_dir.join(
            source
                .as_ref()
                .to_path_buf()
                .strip_prefix(&self.source_dir)?,
        ))
    }

    pub fn add_file(&self, source: &dyn AsRef<Path>) -> Result<()> {
        let mut encoder = self.create_writer(source)?;

        let mut data = Vec::new();
        File::open(source)?.read_to_end(&mut data)?;

        encoder.write_all(&data[..])?;

        Ok(())
    }

    pub fn add_directory(&self, source: &dyn AsRef<Path>) -> Result<()> {
        fs::create_dir(self.out_dir(source)?)?;

        Ok(())
    }

    pub fn add_new_file(&self, source: &dyn AsRef<Path>, data: &mut dyn Read) -> Result<()> {
        let mut encoder = self.create_writer(source)?;

        let mut data_buf = Vec::new();
        data.read_to_end(&mut data_buf)?;

        encoder.write_all(&data_buf[..])?;

        Ok(())
    }
}

pub fn write_files_with_wd(writer: &BackupWriter, from_trait: &dyn AsRef<Path>) -> Result<()> {
    let from = from_trait.as_ref().to_path_buf();

    if from.is_dir() {
        writer.add_directory(&from)?;

        for item in fs::read_dir(&from)? {
            let path = item?.file_name();
            let path_str = path.to_str().unwrap();

            if &path_str.chars().take(1).collect::<String>() == "." {
                continue;
            }

            if &path_str.chars().take(7).collect::<String>() == "__hash_" {
                return Err(Error::msg(format!(
                    "File names may not start with __hash_ (this'd break change detection): {}",
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
