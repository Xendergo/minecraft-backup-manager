use crate::backup::backup_writer::write_files;
use crate::utils::BackupsFolder;
use crate::Command;
use anyhow::{Error, Result};
use chrono::{Datelike, Timelike, Utc};
use clap::ArgMatches;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tar::Builder;

pub struct Backup();

pub struct BackupArgs {
    name: String,
}

impl Command<'_> for Backup {
    type ArgsType = BackupArgs;

    fn parse_args(args: ArgMatches) -> Result<Self::ArgsType> {
        Ok(BackupArgs {
            name: match args.value_of("name") {
                Some(v) => v.to_string(),
                None => {
                    let t = Utc::now();

                    format!(
                        "{}-{}-{}_{}-{}-{}",
                        t.year(),
                        t.month(),
                        t.day(),
                        t.hour(),
                        t.minute(),
                        t.second()
                    )
                }
            } + ".tar",
        })
    }

    fn run_command(args: Self::ArgsType) -> Result<()> {
        let backups = BackupsFolder::get()?;
        let backups_dir = backups.dir();
        let new_backup_path = backups_dir.join(&args.name);
        let mc_dir = backups_dir.parent().unwrap().to_path_buf();

        println!("Storing the backup in {}", &args.name);

        if backups_dir.join(&args.name).is_file() {
            return Err(Error::msg("A backup with this name already exists"));
        }

        println!("Copying, hashing, and compressing files");

        let archive_file = File::create(&new_backup_path)?;

        let mut encoder = GzEncoder::new(archive_file, Compression::best());

        make_backup(mc_dir, &mut encoder)?;

        encoder.finish()?;

        println!("Backup completed");

        Ok(())
    }
}

fn make_backup(mc_dir: PathBuf, encoder: &mut impl Write) -> Result<u64> {
    let mut archive = Builder::new(encoder);
    let hash = write_files(&mc_dir, &mut archive)?;
    archive.finish()?;
    Ok(hash)
}
