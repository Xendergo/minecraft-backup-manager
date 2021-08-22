use std::{fs::File, io::{self, ErrorKind}, path::Path};

pub fn option_open(path: impl AsRef<Path>) -> io::Result<Option<File>> {
    let opened = File::open(path);

    match opened {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(e)
        }
    }
}