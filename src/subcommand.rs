use clap::ArgMatches;

pub trait Command {
    fn run_command(args: ArgMatches);
}
