mod cli;
mod lua;

use std::{error::Error, path::PathBuf};

const DEFAULT_CONFIG_DIRECTORY: &str = "/etc/carbide";

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::get_matches();
    match matches.subcommand() {
        Some(("switch", subcommand)) => {
            let config_directory = PathBuf::from(
                subcommand
                    .get_one::<String>("config-directory")
                    .unwrap_or(&String::from(DEFAULT_CONFIG_DIRECTORY)),
            );

            let actions = lua::load(config_directory)?;
            dbg!(actions);
        }
        Some(("generation", subcommand)) => match subcommand.subcommand() {
            Some(("list", _)) => todo!(),
            Some(("delete", _)) => todo!(),
            Some(("clean", _)) => todo!(),
            _ => {}
        },
        _ => {}
    }

    Ok(())
}
