use anyhow::Result;
use clap::ArgMatches;

pub trait Command {
    fn run_command(args: ArgMatches) -> Result<()>;
}
