use anyhow::Result;
use clap::ArgMatches;

pub trait Command<'a> {
    type ArgsType;

    fn parse_args(args: ArgMatches<'a>) -> Result<Self::ArgsType>;
    fn run_command(args: Self::ArgsType) -> Result<()>;
}

pub fn run_command<Cmd>(args: ArgMatches) -> Result<()>
where
    Cmd: for<'a> Command<'a>,
{
    Cmd::run_command(Cmd::parse_args(args)?)
}
