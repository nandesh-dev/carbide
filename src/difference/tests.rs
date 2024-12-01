use std::path::PathBuf;

use chrono::Local;

use crate::{
    difference::{
        self,
        models::{Action, Difference, File},
    },
    generations,
};

#[test]
fn differ_generations_files() {
    let initial_generation = generations::models::Generation {
        id: 0,
        creation_datetime: Local::now(),
        files: vec![
            generations::models::File {
                path: PathBuf::from("set_and_"),
                content: Some(String::from("Hello World")),
            },
            generations::models::File {
                path: PathBuf::from("set_and_delete"),
                content: Some(String::from("Hello World")),
            },
            generations::models::File {
                path: PathBuf::from("set_and_update"),
                content: Some(String::from("Initial Content")),
            },
        ],
        scripts: vec![],
    };

    let final_generation = generations::models::Generation {
        id: 1,
        creation_datetime: Local::now(),
        files: vec![
            generations::models::File {
                path: PathBuf::from("set_and_delete"),
                content: None,
            },
            generations::models::File {
                path: PathBuf::from("set_and_update"),
                content: Some(String::from("Final Content")),
            },
            generations::models::File {
                path: PathBuf::from("_and_create"),
                content: Some(String::from("New Content")),
            },
        ],
        scripts: vec![],
    };

    assert_eq!(
        difference::differ_generations(&initial_generation, &final_generation),
        Difference {
            actions: vec![
                Action::File(File::Delete {
                    path: PathBuf::from("set_and_delete"),
                }),
                Action::File(File::Update {
                    path: PathBuf::from("set_and_update"),
                    content: String::from("Final Content")
                }),
                Action::File(File::Create {
                    path: PathBuf::from("_and_create"),
                    content: String::from("New Content")
                }),
                Action::File(File::Delete {
                    path: PathBuf::from("set_and_"),
                })
            ]
        }
    )
}

#[test]
fn differ_generations_scripts() {
    let initial_generation = generations::models::Generation {
        id: 0,
        creation_datetime: Local::now(),
        files: vec![],
        scripts: vec![
            generations::models::Script {
                install: vec![String::from("sudo apt-get install ffmpeg")],
                update: vec![String::from("sudo apt-get update ffmpeg")],
                uninstall: vec![String::from("sudo apt-get uninstall ffmpeg")],
            },
            generations::models::Script {
                install: vec![String::from("sudo apt-get install docker")],
                update: vec![String::from("sudo apt-get update docker")],
                uninstall: vec![String::from("sudo apt-get uninstall docker")],
            },
        ],
    };

    let final_generation = generations::models::Generation {
        id: 1,
        creation_datetime: Local::now(),
        files: vec![],
        scripts: vec![generations::models::Script {
            install: vec![String::from("sudo apt-get install ffmpeg_2")],
            update: vec![String::from("sudo apt-get update ffmpeg")],
            uninstall: vec![String::from("sudo apt-get uninstall ffmpeg")],
        }],
    };

    assert_eq!(
        difference::differ_generations(&initial_generation, &final_generation),
        Difference {
            actions: vec![
                Action::Script(vec![String::from("sudo apt-get uninstall ffmpeg"),]),
                Action::Script(vec![String::from("sudo apt-get uninstall docker")]),
                Action::Script(vec![String::from("sudo apt-get install ffmpeg_2"),]),
            ]
        }
    )
}
