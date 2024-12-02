mod cli;
mod difference;
mod generations;
mod lua;

use std::{
    error::Error,
    fs::{remove_file, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use chrono::Local;

const DEFAULT_CONFIG_DIRECTORY: &str = "/etc/carbide";
const DEFAULT_DATA_DIRECTORY: &str = "/var/lib/carbide";

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

            let config = lua::parse_config(&config_directory)?;

            let previous_generation = match generations::read_last_generation(&data_directory) {
                Ok(previous_generation) => previous_generation,
                Err(err) => match err.kind() {
                    io::ErrorKind::NotFound => generations::models::Generation::new(),
                    _ => return Err(Box::new(err)),
                },
            };

            let current_generation = generations::models::Generation::from_lua_config(
                &config,
                previous_generation.id + 1,
                &Local::now(),
            )?;

            current_generation
                .write(&data_directory.join(format!("carbide-{}", previous_generation.id + 1)))?;

            let difference =
                difference::differ_generations(&previous_generation, &current_generation);

            for action in &difference.actions {
                match action {
                    difference::models::Action::File(file) => match file {
                        difference::models::File::Create { path, content } => {
                            println!("[ Creating File ] {}", path.display());

                            let mut file = File::create(&path)?;
                            file.write_all(content.as_bytes())?;
                        }
                        difference::models::File::Update { path, content } => {
                            println!("[ Updating File ] {}", path.display());

                            let mut file =
                                OpenOptions::new().write(true).truncate(true).open(&path)?;
                            file.write_all(content.as_bytes())?;
                        }
                        difference::models::File::Delete { path } => {
                            println!("[ Deleting File ] {}", path.display());

                            remove_file(&path)?;
                        }
                    },
                    difference::models::Action::Script(script) => {
                        for command in script {
                            println!("[ Running Command ]");
                            println!("{}", command)
                        }
                    }
                }
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
