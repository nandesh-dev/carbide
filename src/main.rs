mod cli;
mod lua;

use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::get_matches();
    match matches.subcommand() {
        Some(("switch", subcommand)) => {
            let config_directory = PathBuf::from(
                subcommand
                    .get_one::<String>("config-directory")
                    .expect("Missing config directory"),
            );

            let actions = lua::load(config_directory)?;
            dbg!(actions);
        }
        Some(("generation", subcommand)) => match subcommand.subcommand() {
            Some(("list", _)) => todo!(),
            Some(("delete", subcommand)) => todo!(),
            Some(("clean", _)) => todo!(),
            _ => todo!(),
        },
        _ => todo!(),
    }

    Ok(())
}
