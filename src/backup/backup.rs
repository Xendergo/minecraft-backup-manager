use crate::utils::copy_and_hash;
use crate::utils::BackupsFolder;
use crate::Command;
use anyhow::{Error, Result};
use chrono::{Datelike, Timelike, Utc};
use clap::ArgMatches;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use tar::Builder;

pub struct Backup();

impl Command for Backup {
    fn run_command(args: ArgMatches) -> Result<()> {
        let name = match args.value_of("name") {
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
        } + ".tar.gz";

        let backups = BackupsFolder::get()?;
        let backups_dir = backups.dir();
        let new_backup_path = backups_dir.join(&name);
        let mc_dir = backups_dir.parent().unwrap().to_path_buf();

        if backups_dir.join(name).is_file() {
            return Err(Error::msg("A backup with this name already exists"));
        }

        println!("Copying, hashing, and compressing files");

        let archive_file = File::create(new_backup_path)?;

        let mut encoder = GzEncoder::new(archive_file, Compression::best());

        {
            let mut archive = Builder::new(&mut encoder);
            copy_and_hash(&mc_dir, &mut archive)?;
        }

        encoder.finish()?;

        println!("Backup completed");

        Ok(())
    }
}
