use crate::utils::TempFile;
use std::io;
use std::io::Write;
use std::path::Path;
use tar::Builder;

pub trait WriteNewFile {
    fn append_new_file(&mut self, path: impl AsRef<Path>, data: &[u8]) -> io::Result<()>;
}

impl<W: Write> WriteNewFile for Builder<W> {
    fn append_new_file(&mut self, path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
        let mut temp_file = TempFile::new();
        temp_file.get_writeonly()?.write(data)?;
        self.append_file(path, &mut temp_file.get_readonly()?)
    }
}
