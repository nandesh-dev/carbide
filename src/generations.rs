use regex::Regex;
use std::{fs, io, path::PathBuf};

pub fn read_generations(path: &PathBuf) -> io::Result<Vec<models::Generation>> {
    let mut generations: Vec<models::Generation> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name_regex = Regex::new(r"^carbide-\d+$").unwrap();

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if !file_name_regex.is_match(file_name) {
                continue;
            }

            generations.push(models::Generation::from_file(path)?)
        }
    }

    generations.sort_by_key(|g| g.id);

    Ok(generations)
}

pub fn read_last_generation(path: &PathBuf) -> io::Result<models::Generation> {
    let mut highest_id = -1;

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name_regex = Regex::new(r"^carbide-(\d+)$").unwrap();

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(captures) = file_name_regex.captures(file_name) {
                let id = captures.get(1).unwrap();

                let id = i32::from_str_radix(id.as_str(), 10).map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Error parsing generation id: {}", err),
                    )
                })?;

                if id > highest_id {
                    highest_id = id
                }
            }
            if !file_name_regex.is_match(file_name) {
                continue;
            }

            file_name_regex.captures(file_name);

            return Ok(models::Generation::from_file(path)?);
        }
    }

    if highest_id == -1 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No generation files found",
        ));
    }

    Ok(models::Generation::from_file(
        path.join(PathBuf::from(format!("carbide-{}", highest_id))),
    )?)
}

pub mod models {
    use crate::lua;
    use chrono::{DateTime, Local};
    use serde::{Deserialize, Serialize};
    use std::{fs, io, path::PathBuf};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Generation {
        pub id: i32,
        pub creation_datetime: DateTime<Local>,
        pub files: Vec<File>,
        pub scripts: Vec<Script>,
    }

    impl Generation {
        pub fn from_file(path: PathBuf) -> io::Result<Self> {
            let file = fs::File::open(path)?;
            bincode::deserialize_from(file).map_err(|err| {
                io::Error::new(io::ErrorKind::Other, format!("Bincode Error: {}", err))
            })
        }

        pub fn from_lua_config(
            id: i32,
            creation_datetime: DateTime<Local>,
            config: lua::models::Config,
        ) -> Result<Self, String> {
            let mut files = Vec::<File>::new();
            let mut scripts = Vec::<Script>::new();

            for action in config.actions {
                match action {
                    lua::models::Action::Script(script) => {
                        scripts.push(Script {
                            install: script.install,
                            uninstall: script.uninstall,
                            update: script.update,
                        });
                    }
                    lua::models::Action::File(file) => match file {
                        lua::models::File::Set { path, content } => {
                            for existing_file in files.iter() {
                                if path == existing_file.path {
                                    return Err(format!(
                                        "Duplicate file with path: {}",
                                        path.display()
                                    ));
                                }
                            }

                            files.push(File {
                                path,
                                content: Some(content),
                            });
                        }
                        lua::models::File::Append { path, content } => {
                            let mut file_exists = false;

                            for existing_file in files.iter_mut() {
                                if path == existing_file.path {
                                    existing_file.content = match &existing_file.content {
                                        Some(original_content) => {
                                            Some(format!("{}\n{}", original_content, content))
                                        }
                                        None => Some(content.clone()),
                                    };

                                    file_exists = true;
                                    break;
                                }
                            }

                            if !file_exists {
                                files.push(File {
                                    path,
                                    content: Some(content),
                                });
                            }
                        }
                        lua::models::File::Delete { path } => {
                            for existing_file in files.iter() {
                                if path == existing_file.path {
                                    return Err(format!(
                                        "Cannot set and delete a file in the same generation: {}",
                                        path.display()
                                    ));
                                }
                            }

                            files.push(File {
                                path,
                                content: None,
                            });
                        }
                    },
                }
            }

            Ok(Self {
                id,
                creation_datetime,
                files,
                scripts,
            })
        }

        pub fn write(self, path: PathBuf) -> io::Result<()> {
            let file = fs::File::create(path)?;
            bincode::serialize_into(file, &self).map_err(|err| {
                io::Error::new(io::ErrorKind::Other, format!("Bincode error: {}", err))
            })
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct File {
        pub path: PathBuf,
        pub content: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Script {
        pub install: Vec<String>,
        pub update: Vec<String>,
        pub uninstall: Vec<String>,
    }
}
