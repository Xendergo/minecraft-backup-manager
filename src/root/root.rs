use crate::backup::Backup;
use crate::subcommand::Command;
use clap::ArgMatches;

pub struct Root();

impl Command for Root {
    fn run_command(args: ArgMatches) {
        let subcommand = args.subcommand.unwrap();
        let name: &str = &subcommand.name;

        match name {
            "backup" => Backup::run_command(subcommand.matches),
            _ => unreachable!(),
        };
    }
}
