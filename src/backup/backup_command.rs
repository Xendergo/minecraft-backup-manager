use crate::backup::Backup;
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

pub struct BackupCommand();

enum BackupType {
    Full,
    Partial,
}

pub struct BackupArgs {
    name: String,
    backup_type: BackupType,
}

impl Command<'_> for BackupCommand {
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
            } + ".tar.gz",
            backup_type: match args.value_of("type") {
                Some("full") => BackupType::Full,
                Some("partial") => BackupType::Partial,
                _ => BackupType::Partial,
            },
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

        make_backup(mc_dir, &mut encoder, backups, args.name)?;

        encoder.finish()?;

        println!("Backup completed");

        Ok(())
    }
}

fn make_backup(
    mc_dir: PathBuf,
    encoder: &mut impl Write,
    backups_dir: BackupsFolder,
    name: String,
) -> Result<Backup> {
    let mut archive = Builder::new(encoder);
    let backup = Backup::create(&mc_dir, &mut archive, backups_dir, name)?;
    archive.finish()?;
    Ok(backup)
}
