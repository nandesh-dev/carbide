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

            println!(
                "[ Stage 1 ] ( Data Directory ) {}",
                data_directory.display()
            );

            let config_directory = PathBuf::from(
                subcommand
                    .get_one::<String>("config-directory")
                    .unwrap_or(&String::from(DEFAULT_CONFIG_DIRECTORY)),
            );

            println!(
                "[ Stage 1 ] ( Config Directory ) {}",
                config_directory.display()
            );

            println!("[ Stage 1 ] ( Loading Config )");
            let config = lua::parse_config(&config_directory)?;

            println!("[ Stage 2 ] ( Reading Last Generation )");
            let previous_generation = match generations::read_last_generation(&data_directory) {
                Ok(previous_generation) => previous_generation,
                Err(err) => match err.kind() {
                    io::ErrorKind::NotFound => generations::models::Generation::new(),
                    _ => return Err(Box::new(err)),
                },
            };

            println!("[ Stage 2 ] ( Generating Current Generation )");
            let current_generation = generations::models::Generation::from_lua_config(
                &config,
                previous_generation.id + 1,
                &Local::now(),
            )?;

            println!("[ Stage 2 ] ( Saving Current Generation )");
            current_generation
                .write(&data_directory.join(format!("carbide-{}", previous_generation.id + 1)))?;

            println!("[ Stage 3 ] ( Calculating Differences )");
            let difference =
                difference::differ_generations(&previous_generation, &current_generation);

            if difference.actions.len() > 0 {
                println!("[ Stage 4 ] ( Applying Differences )");
            } else {
                println!("[ Stage 4 ] ( No Differences Found )");
            }

            for action in &difference.actions {
                match action {
                    difference::models::Action::File(file) => match file {
                        difference::models::File::Create { path, content } => {
                            println!("[ Stage 4 ] ( Creating File ) {}", path.display());

                            let mut file = File::create(&path)?;
                            file.write_all(content.as_bytes())?;
                        }
                        difference::models::File::Update { path, content } => {
                            println!("[ Stage 4 ] ( Updating File ) {}", path.display());

                            let mut file =
                                OpenOptions::new().write(true).truncate(true).open(&path)?;
                            file.write_all(content.as_bytes())?;
                        }
                        difference::models::File::Delete { path } => {
                            println!("[ Stage 4 ] ( Deleting File ) {}", path.display());

                            remove_file(&path)?;
                        }
                    },
                    difference::models::Action::Script(script) => {
                        for command in script {
                            println!("[ Stage 4 ] ( Running Command ) {}", command);
                        }
                    }
                }
            }

            println!("[ Stage 5 ] ( Complete )");
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
