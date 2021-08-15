use crate::backup::BackupCommand;
use crate::run_command;
use crate::subcommand::Command;
use anyhow::Result;
use clap::ArgMatches;

pub struct Root();

pub struct RootArgs<'a> {
    name: String,
    matches: ArgMatches<'a>,
}

impl<'a> Command<'a> for Root {
    type ArgsType = RootArgs<'a>;

    fn parse_args(args: ArgMatches<'a>) -> Result<Self::ArgsType> {
        let subcommand = args.subcommand.unwrap();

        Ok(RootArgs {
            name: subcommand.name,
            matches: subcommand.matches,
        })
    }

    fn run_command(args: Self::ArgsType) -> Result<()> {
        match &args.name[..] {
            "backup" => run_command::<BackupCommand>(args.matches)?,
            _ => unreachable!(),
        };

        Ok(())
    }
}
