use clap::{Arg, ArgMatches, Command};

pub fn get_matches() -> ArgMatches {
    Command::new("carbide")
        .subcommand(
            Command::new("switch")
                .about("Switch to the latest config")
                .arg(
                    Arg::new("config-directory")
                        .long("config-directory")
                        .short('c')
                        .help("Set configuration directory path"),
                ),
        )
        .subcommand(
            Command::new("generation")
                .subcommand(Command::new("list").about("List available generations"))
                .subcommand(
                    Command::new("delete")
                        .about("Delete a generation")
                        .arg(Arg::new("generation-id").help("Generation ID")),
                )
                .subcommand(Command::new("clean").about("Deletes old generations"))
                .subcommand_required(true),
        )
        .disable_help_subcommand(true)
        .subcommand_required(true)
        .get_matches()
}
