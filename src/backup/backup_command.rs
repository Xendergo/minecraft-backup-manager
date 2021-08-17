use crate::backup::Backup;
use crate::utils::BackupsFolder;
use crate::Command;
use anyhow::{Error, Result};
use chrono::{Datelike, Timelike, Utc};
use clap::ArgMatches;

pub struct BackupCommand();

pub enum BackupType {
    Full,
    Partial,
}

pub struct BackupArgs {
    pub name: String,
    pub backup_type: BackupType,
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
            } + ".zip",
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
        let mc_dir = backups_dir.parent().unwrap().to_path_buf();

        println!("Storing the backup in {}", &args.name);

        if backups_dir.join(&args.name).is_file() {
            return Err(Error::msg("A backup with this name already exists"));
        }

        println!("Copying, hashing, and compressing files");

        Backup::create(&mc_dir, backups, &args)?;

        println!("Backup completed");

        Ok(())
    }
}
