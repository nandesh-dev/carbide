use std::path::PathBuf;

use assert_fs::prelude::*;

use crate::lua::models::{Action, Config, File, Script};

use super::parse_config;

#[test]
fn parse_config_script() {
    let config_directory = assert_fs::TempDir::new().unwrap();
    let init_lua_file = config_directory.child("init.lua");
    init_lua_file.write_str("carbide.script({ \"sudo apt-get install neovim\"}, {}, { \"sudo apt-get uninstall neovim\" })").unwrap();

    let config = parse_config(&PathBuf::from(config_directory.path())).unwrap();

    assert_eq!(
        config,
        Config {
            actions: vec![Action::Script(Script {
                install: vec![String::from("sudo apt-get install neovim")],
                update: vec![],
                uninstall: vec![String::from("sudo apt-get uninstall neovim")],
            })]
        }
    )
}

#[test]
fn parse_config_file_set() {
    let config_directory = assert_fs::TempDir::new().unwrap();
    let init_lua_file = config_directory.child("init.lua");
    init_lua_file
        .write_str("carbide.file.set(\"/etc/neovim/init.lua\", \"print(\\\"Hello World\\\")\")")
        .unwrap();

    let config = parse_config(&PathBuf::from(config_directory.path())).unwrap();

    assert_eq!(
        config,
        Config {
            actions: vec![Action::File(File::Set {
                path: PathBuf::from("/etc/neovim/init.lua"),
                content: String::from("print(\"Hello World\")")
            })]
        }
    )
}

#[test]
fn parse_config_file_append() {
    let config_directory = assert_fs::TempDir::new().unwrap();
    let init_lua_file = config_directory.child("init.lua");
    init_lua_file
        .write_str(
            "carbide.file.append(\"/etc/neovim/init.lua\", \"print(\\\"Hello World\\\")\")
carbide.file.append(\"/etc/neovim/init.lua\", \"print(\\\"Hello World 2\\\")\")",
        )
        .unwrap();

    let config = parse_config(&PathBuf::from(config_directory.path())).unwrap();

    assert_eq!(
        config,
        Config {
            actions: vec![
                Action::File(File::Append {
                    path: PathBuf::from("/etc/neovim/init.lua"),
                    content: String::from("print(\"Hello World\")")
                }),
                Action::File(File::Append {
                    path: PathBuf::from("/etc/neovim/init.lua"),
                    content: String::from("print(\"Hello World 2\")")
                })
            ]
        }
    )
}

#[test]
fn parse_config_file_delete() {
    let config_directory = assert_fs::TempDir::new().unwrap();
    let init_lua_file = config_directory.child("init.lua");
    init_lua_file
        .write_str("carbide.file.delete(\"/etc/neovim/init.lua\")")
        .unwrap();

    let config = parse_config(&PathBuf::from(config_directory.path())).unwrap();

    assert_eq!(
        config,
        Config {
            actions: vec![Action::File(File::Delete {
                path: PathBuf::from("/etc/neovim/init.lua"),
            })]
        }
    )
}
