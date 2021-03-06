mod backup;
mod root;
mod subcommand;
mod utils;

#[macro_use]
extern crate clap;
use crate::subcommand::run_command;
use anyhow::Result;
use clap::AppSettings;
use root::Root;
use subcommand::Command;

fn main() -> Result<()> {
    let args = clap_app!(("Minecraft backup manager") =>
        (version: crate_version!())
        (author: "Xendergo")
        (about: "Manages backups for your minecraft worlds")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (@subcommand backup =>
            (about: "Backup your world")
            (@arg name: -n --name +takes_value "The name of the new backup")
            (@arg type: -t --type +takes_value possible_values(&["full", "partial"]) "whether the backup should take a backup of all the files or only the ones that have changed.\nUsing `partial` doesn't effect the ability to restore data in any way, unless previous backups are altered.")
        )
        (@subcommand restore =>
            (about: "Restore your world from a previous backup, backing up beforehand is reccommended")
            (@arg name: -n --name +takes_value "The name of the backup to restore, restores the most recent by default")
        )
    )
    .get_matches();

    run_command::<Root>(args)
}
