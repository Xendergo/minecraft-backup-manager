use anyhow::Error;
use anyhow::Result;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;

pub struct BackupWriter {
    backup: ZipWriter<File>,
    source_dir: PathBuf,
}

impl BackupWriter {
    pub fn new(
        source_dir: &dyn AsRef<Path>,
        backups_dir: &dyn AsRef<Path>,
        name: &str,
    ) -> Result<BackupWriter> {
        let file = backups_dir.as_ref().to_path_buf().join(name);
        Ok(BackupWriter {
            source_dir: source_dir.as_ref().to_path_buf(),
            backup: ZipWriter::new(File::create(file)?),
        })
    }

    fn out_dir(&self, source: &dyn AsRef<Path>) -> Result<PathBuf> {
        Ok(source
            .as_ref()
            .to_path_buf()
            .strip_prefix(&self.source_dir)?
            .to_path_buf())
    }

    pub fn add_file(&mut self, source: &dyn AsRef<Path>) -> Result<()> {
        let dir = self.out_dir(source)?;

        let mut data = Vec::new();
        File::open(source)?.read_to_end(&mut data)?;

        self.backup
            .start_file(dir.to_str().unwrap(), FileOptions::default())?;
        self.backup.write_all(&mut data[..])?;

        Ok(())
    }

    pub fn add_directory(&mut self, source: &dyn AsRef<Path>) -> Result<()> {
        let dir = self.out_dir(source)?;

        self.backup
            .add_directory(dir.to_str().unwrap(), FileOptions::default())?;

        Ok(())
    }

    pub fn add_new_file(&mut self, source: &dyn AsRef<Path>, data: &mut dyn Read) -> Result<()> {
        let dir = self.out_dir(source)?;
        let mut data_buf = Vec::new();
        data.read_to_end(&mut data_buf)?;

        self.backup
            .start_file(dir.to_str().unwrap(), FileOptions::default())?;
        self.backup.write_all(&data_buf[..])?;

        Ok(())
    }
}

pub fn write_files_with_wd(writer: &mut BackupWriter, from_trait: &dyn AsRef<Path>) -> Result<()> {
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
