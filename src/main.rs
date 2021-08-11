mod backup;
mod root;
mod subcommand;

#[macro_use]
extern crate clap;
use clap::AppSettings;
use root::Root;
use subcommand::Command;

fn main() {
    let args = clap_app!(("Minecraft backup manager") =>
        (version: crate_version!())
        (author: "Xendergo")
        (about: "Manages backups for your minecraft worlds")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (@subcommand backup =>
            (about: "Backup your world")
            (@arg name: -n --name +takes_value "The name of the backup")
        )
    )
    .get_matches();

    Root::run_command(args)
}
