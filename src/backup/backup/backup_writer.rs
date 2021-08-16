use crate::utils::TempFile;
use anyhow::Error;
use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::Write;
use std::path::PathBuf;
use tar::Builder;

pub fn write_files_with_wd(
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
