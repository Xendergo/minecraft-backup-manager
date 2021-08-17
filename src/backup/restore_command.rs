use crate::backup::backup::BackupReader;
use crate::utils::BackupsFolder;
use crate::Command;
use anyhow::Error;
use anyhow::Result;
use clap::ArgMatches;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs;

pub struct RestoreCommand();

pub struct RestoreArgs {
    path: PathBuf,
}

impl Command<'_> for RestoreCommand {
    type ArgsType = RestoreArgs;

    fn parse_args(args: ArgMatches) -> Result<Self::ArgsType> {
        let backups = BackupsFolder::get()?;

        let path = match args.value_of("name") {
            Some(v) => backups.join(v),
            None => match backups.current_backup()? {
                Some(v) => v,
                None => match backups.all_backups()?.nth(0) {
                    Some(_) => return Err(Error::msg("The file marking the most recent backup is missing or invalid, specify which backup you want to restore with --name")),
                    None => return Err(Error::msg("There are no backups to restore")),
                },
            },
        }.with_extension("zip");

        if !path.exists() {
            return Err(Error::msg(format!(
                "There's no backup with that name {}",
                path.file_name().unwrap().to_string_lossy()
            )));
        }

        Ok(RestoreArgs { path: path })
    }

    fn run_command(args: Self::ArgsType) -> Result<()> {
        let mut backup = BackupReader::new(args.path)?;
        let backups_folder = BackupsFolder::get()?;
        let folder_to_restore_to = backups_folder.parent().unwrap();

        let file_names = backup
            .file_names()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();

        for file_name in file_names {
            let mut file = backup.get_file(&file_name)?;

            let path = folder_to_restore_to.join(file_name);

            if file.is_dir() {
                if !path.exists() {
                    fs::create_dir(path)?;
                }

                continue;
            }

            let mut out = File::create(path)?;

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            out.write_all(&data)?;
        }

        Ok(())
    }
}
