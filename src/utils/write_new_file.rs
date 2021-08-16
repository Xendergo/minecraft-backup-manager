use crate::utils::TempFile;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use tar::Builder;

pub trait WriteNewFile {
    fn append_new_file(&mut self, path: impl AsRef<Path>, data: &[u8]) -> io::Result<()>;
}

impl<W: Write> WriteNewFile for Builder<W> {
    fn append_new_file(&mut self, path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
        let mut temp_file = TempFile::new()?;
        temp_file.write(data)?;
        self.append_file(path, &mut File::open(temp_file)?)
    }
}
