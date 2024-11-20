mod cli;
mod generations;
mod lua;

use std::{error::Error, io, path::PathBuf};

use chrono::Local;

const DEFAULT_CONFIG_DIRECTORY: &str = "/etc/carbide";
const DEFAULT_DATA_DIRECTORY: &str = "/local/carbide";

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::get_matches();
    match matches.subcommand() {
        Some(("switch", subcommand)) => {
            let data_directory = PathBuf::from(
                subcommand
                    .get_one::<String>("data-directory")
                    .unwrap_or(&String::from(DEFAULT_DATA_DIRECTORY)),
            );
            let config_directory = PathBuf::from(
                subcommand
                    .get_one::<String>("config-directory")
                    .unwrap_or(&String::from(DEFAULT_CONFIG_DIRECTORY)),
            );

            let config = lua::parse_config(config_directory)?;
            let current_generation =
                generations::models::Generation::from_lua_config(0, Local::now(), config)?;

            match generations::read_last_generation(&data_directory) {
                Ok(last_generation) => {
                    current_generation.write(
                        data_directory.join(format!("carbide-{}", last_generation.id + 1)),
                    )?;
                }
                Err(err) => match err.kind() {
                    io::ErrorKind::NotFound => {
                        current_generation.write(data_directory.join("carbide-0"))?;
                    }
                    _ => return Err(Box::new(err)),
                },
            }
        }
        Some(("generation", subcommand)) => match subcommand.subcommand() {
            Some(("list", subcommand)) => {
                let data_directory = PathBuf::from(
                    subcommand
                        .get_one::<String>("data-directory")
                        .unwrap_or(&String::from(DEFAULT_DATA_DIRECTORY)),
                );
                for generation in generations::read_generations(&data_directory)? {
                    println!(
                        "{} : {} : {}",
                        generation.id,
                        generation.creation_datetime.format("%Y-%m-%d %H:%M:%S"),
                        data_directory
                            .join(format!("carbide-{}", generation.id))
                            .display()
                    )
                }
            }
            Some(("delete", _)) => todo!(),
            Some(("clean", _)) => todo!(),
            _ => {}
        },
        _ => {}
    }

    Ok(())
}
