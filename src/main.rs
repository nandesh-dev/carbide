mod cli;
mod generations;
mod lua;

use std::{error::Error, path::PathBuf};

const DEFAULT_CONFIG_DIRECTORY: &str = "/etc/carbide";
const DEFAULT_STORAGE_DIRECTORY: &str = "/local/carbide";

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::get_matches();
    match matches.subcommand() {
        Some(("switch", subcommand)) => {
            let config_directory = PathBuf::from(
                subcommand
                    .get_one::<String>("config-directory")
                    .unwrap_or(&String::from(DEFAULT_CONFIG_DIRECTORY)),
            );

            let config = lua::parse_config(config_directory)?;
            let generation = generations::models::Generation::from_lua_config(0, config)?;

            dbg!(&generation);

            generation.write(PathBuf::from("temp/out"))?;

            let generation2 =
                generations::models::Generation::from_file(PathBuf::from("temp/out"))?;

            dbg!(generation2);
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
