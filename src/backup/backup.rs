use crate::Command;
use clap::ArgMatches;

pub struct Backup();

impl Command for Backup {
    fn run_command(args: ArgMatches) {
        println!("{:?}", args)
    }
}
