use crate::lua;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Generation {
    pub id: i32,
    pub creation_datetime: DateTime<Local>,
    pub files: Vec<File>,
    pub scripts: Vec<Script>,
}

impl Generation {
    pub fn from_file(path: &PathBuf) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        bincode::deserialize_from(file)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("Bincode Error: {}", err)))
    }

    pub fn from_lua_config(
        config: &lua::models::Config,
        id: i32,
        creation_datetime: &DateTime<Local>,
    ) -> Result<Self, String> {
        let mut files = Vec::<File>::new();
        let mut scripts = Vec::<Script>::new();

        for action in &config.actions {
            match action {
                lua::models::Action::Script(script) => {
                    scripts.push(Script {
                        install: script.install.clone(),
                        uninstall: script.uninstall.clone(),
                        update: script.update.clone(),
                    });
                }
                lua::models::Action::File(file) => match file {
                    lua::models::File::Set { path, content } => {
                        for existing_file in files.iter() {
                            if *path == existing_file.path {
                                return Err(format!(
                                    "Duplicate file with path: {}",
                                    path.display()
                                ));
                            }
                        }

                        files.push(File {
                            path: path.to_path_buf(),
                            content: Some(content.to_string()),
                        });
                    }
                    lua::models::File::Append { path, content } => {
                        let mut file_exists = false;

                        for existing_file in files.iter_mut() {
                            if *path == existing_file.path {
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
                                path: path.to_path_buf(),
                                content: Some(content.to_string()),
                            });
                        }
                    }
                    lua::models::File::Delete { path } => {
                        for existing_file in files.iter() {
                            if *path == existing_file.path {
                                return Err(format!(
                                    "Cannot set and delete a file in the same generation: {}",
                                    path.display()
                                ));
                            }
                        }

                        files.push(File {
                            path: path.to_path_buf(),
                            content: None,
                        });
                    }
                },
            }
        }

        Ok(Self {
            id,
            creation_datetime: *creation_datetime,
            files,
            scripts,
        })
    }

    pub fn write(&self, path: &PathBuf) -> io::Result<()> {
        let file = fs::File::create(path)?;
        bincode::serialize_into(file, &self)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("Bincode error: {}", err)))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct File {
    pub path: PathBuf,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Script {
    pub install: Vec<String>,
    pub update: Vec<String>,
    pub uninstall: Vec<String>,
}
