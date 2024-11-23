use std::path::PathBuf;

use chrono::Local;

use crate::generations;
use crate::generations::models::{File, Generation, Script};
use crate::lua;

#[test]
fn generation_read_write() {
    let storage_directory = assert_fs::TempDir::new().unwrap();
    let generation = Generation {
        id: 1,
        creation_datetime: Local::now(),
        files: vec![File {
            path: PathBuf::from("/etc/neovim"),
            content: Some(String::from("Hello World")),
        }],
        scripts: vec![Script {
            install: vec![String::from("sudo apt-get install neovim")],
            update: vec![],
            uninstall: vec![String::from("sudo apt-get uninstall neovim")],
        }],
    };

    generation
        .write(storage_directory.join("carbide-0"))
        .unwrap();

    assert_eq!(
        generation,
        Generation::from_file(storage_directory.join("carbide-0")).unwrap()
    )
}

#[test]
fn generation_from_lua_config() {
    let config = lua::models::Config {
        actions: vec![
            lua::models::Action::Script(lua::models::Script {
                install: vec![String::from("sudo apt-get install carbide")],
                update: vec![],
                uninstall: vec![String::from("sudo apt-get uninstall carbide")],
            }),
            lua::models::Action::File(lua::models::File::Set {
                path: PathBuf::from("/file_set"),
                content: String::from("print(\"Hello World\")"),
            }),
            lua::models::Action::File(lua::models::File::Append {
                path: PathBuf::from("/file_append"),
                content: String::from("print(\"Hello World\")"),
            }),
            lua::models::Action::File(lua::models::File::Append {
                path: PathBuf::from("/file_append"),
                content: String::from("print(\"Hello World 2\")"),
            }),
            lua::models::Action::File(lua::models::File::Delete {
                path: PathBuf::from("/file_delete"),
            }),
        ],
    };

    let creation_datetime = Local::now();
    let generation = Generation::from_lua_config(0, creation_datetime, config).unwrap();

    assert_eq!(
        generation,
        Generation {
            id: 0,
            creation_datetime,
            files: vec![
                File {
                    path: PathBuf::from("/file_set"),
                    content: Some(String::from("print(\"Hello World\")"))
                },
                File {
                    path: PathBuf::from("/file_append"),
                    content: Some(String::from(
                        "print(\"Hello World\")\nprint(\"Hello World 2\")"
                    ))
                },
                File {
                    path: PathBuf::from("/file_delete"),
                    content: None
                },
            ],
            scripts: vec![Script {
                install: vec![String::from("sudo apt-get install carbide")],
                update: vec![],
                uninstall: vec![String::from("sudo apt-get uninstall carbide")],
            }]
        }
    )
}

#[test]
fn read_generations() {
    let storage_directory = assert_fs::TempDir::new().unwrap();
    let generation_0 = Generation {
        id: 0,
        creation_datetime: Local::now(),
        files: vec![],
        scripts: vec![Script {
            install: vec![String::from("sudo apt-get install neovim")],
            update: vec![],
            uninstall: vec![String::from("sudo apt-get uninstall neovim")],
        }],
    };

    generation_0
        .write(storage_directory.join("carbide-0"))
        .unwrap();

    let generation_1 = Generation {
        id: 1,
        creation_datetime: Local::now(),
        files: vec![File {
            path: PathBuf::from("/etc/neovim"),
            content: Some(String::from("Hello World")),
        }],
        scripts: vec![Script {
            install: vec![String::from("sudo apt-get install neovim")],
            update: vec![],
            uninstall: vec![String::from("sudo apt-get uninstall neovim")],
        }],
    };

    generation_1
        .write(storage_directory.join("carbide-1"))
        .unwrap();

    assert_eq!(
        generations::read_generations(&PathBuf::from(storage_directory.path())).unwrap(),
        vec![generation_0, generation_1]
    )
}

#[test]
fn read_last_generation() {
    let storage_directory = assert_fs::TempDir::new().unwrap();
    let generation_0 = Generation {
        id: 0,
        creation_datetime: Local::now(),
        files: vec![],
        scripts: vec![Script {
            install: vec![String::from("sudo apt-get install neovim")],
            update: vec![],
            uninstall: vec![String::from("sudo apt-get uninstall neovim")],
        }],
    };

    generation_0
        .write(storage_directory.join("carbide-0"))
        .unwrap();

    let generation_1 = Generation {
        id: 1,
        creation_datetime: Local::now(),
        files: vec![File {
            path: PathBuf::from("/etc/neovim"),
            content: Some(String::from("Hello World")),
        }],
        scripts: vec![Script {
            install: vec![String::from("sudo apt-get install neovim")],
            update: vec![],
            uninstall: vec![String::from("sudo apt-get uninstall neovim")],
        }],
    };

    generation_1
        .write(storage_directory.join("carbide-1"))
        .unwrap();

    assert_eq!(
        generations::read_last_generation(&PathBuf::from(storage_directory.path())).unwrap(),
        generation_1
    )
}
