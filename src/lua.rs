use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mlua::{Lua, LuaOptions, Result, StdLib, Table};

pub fn parse_config(directory: &PathBuf) -> Result<models::Config> {
    let actions = Arc::new(Mutex::new(Vec::<models::Action>::new()));

    let mlua = Lua::new_with(StdLib::PACKAGE, LuaOptions::new())?;
    let carbide_table = mlua.create_table()?;

    let file_table = mlua.create_table()?;

    let actions_clone = Arc::clone(&actions);
    file_table.set(
        "set",
        mlua.create_function(move |_, (path, content): (String, String)| {
            let mut actions = actions_clone.lock().unwrap();
            actions.push(models::Action::File(models::File::Set {
                path: PathBuf::from(path),
                content,
            }));

            Ok(())
        })?,
    )?;

    let actions_clone = Arc::clone(&actions);
    file_table.set(
        "append",
        mlua.create_function(move |_, (path, content): (String, String)| {
            let mut actions = actions_clone.lock().unwrap();
            actions.push(models::Action::File(models::File::Append {
                path: PathBuf::from(path),
                content,
            }));

            Ok(())
        })?,
    )?;

    let actions_clone = Arc::clone(&actions);
    file_table.set(
        "delete",
        mlua.create_function(move |_, path: String| {
            let mut actions = actions_clone.lock().unwrap();
            actions.push(models::Action::File(models::File::Delete {
                path: PathBuf::from(path),
            }));

            Ok(())
        })?,
    )?;

    carbide_table.set("file", file_table)?;

    let actions_clone = Arc::clone(&actions);
    carbide_table.set(
        "script",
        mlua.create_function(
            move |_, (install, update, uninstall): (Vec<String>, Vec<String>, Vec<String>)| {
                let mut actions = actions_clone.lock().unwrap();
                actions.push(models::Action::Script(models::Script {
                    install,
                    update,
                    uninstall,
                }));

                Ok(())
            },
        )?,
    )?;

    mlua.globals().set("carbide", carbide_table)?;

    let package: Table = mlua.globals().get("package")?;
    let package_path: String = package.get("path")?;
    package.set(
        "path",
        format!(
            "{};{}",
            directory
                .join("?.lua")
                .to_str()
                .expect("Invalid config path")
                .to_string(),
            package_path
        ),
    )?;

    let init_script = fs::read_to_string(
        directory
            .join("init.lua")
            .to_str()
            .expect("Invalid init script path")
            .to_string(),
    )?;
    mlua.load(init_script).set_name("init.lua").exec()?;

    let actions = actions.lock().unwrap().clone();
    Ok(models::Config { actions })
}

pub mod models {
    use std::path::PathBuf;

    use mlua::{FromLua, IntoLua, Lua, Value};

    #[derive(Debug, PartialEq)]
    pub struct Config {
        pub actions: Vec<Action>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Action {
        Script(Script),
        File(File),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Script {
        pub install: Vec<String>,
        pub update: Vec<String>,
        pub uninstall: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum File {
        Set { path: PathBuf, content: String },
        Append { path: PathBuf, content: String },
        Delete { path: PathBuf },
    }

    impl<'lua> IntoLua for Action {
        fn into_lua(self, lua: &Lua) -> mlua::prelude::LuaResult<Value> {
            let table = lua.create_table()?;

            match self {
                Action::File(file) => {
                    table.set("action", "file")?;
                    match file {
                        File::Set { path, content } => {
                            table.set("method", "set")?;
                            table.set("path", path)?;
                            table.set("content", content)?;
                        }
                        File::Append { path, content } => {
                            table.set("method", "append")?;
                            table.set("path", path)?;
                            table.set("content", content)?;
                        }
                        File::Delete { path } => {
                            table.set("method", "delete")?;
                            table.set("path", path)?;
                        }
                    }
                }
                Action::Script(script) => {
                    table.set("action", "script")?;
                    table.set("install", script.install)?;
                    table.set("update", script.update)?;
                    table.set("uninstall", script.uninstall)?;
                }
            }

            Ok(Value::Table(table))
        }
    }

    impl<'lua> FromLua for Action {
        fn from_lua(value: Value, _: &Lua) -> mlua::prelude::LuaResult<Self> {
            let table = value.as_table().unwrap();

            let action: String = table.get("action")?;
            match action.as_str() {
                "file" => {
                    let method: String = table.get("method")?;
                    match method.as_str() {
                        "set" => Ok(Self::File(File::Set {
                            path: table.get("path")?,
                            content: table.get("content")?,
                        })),
                        "append" => Ok(Self::File(File::Set {
                            path: table.get("path")?,
                            content: table.get("content")?,
                        })),
                        "delete" => Ok(Self::File(File::Set {
                            path: table.get("path")?,
                            content: table.get("content")?,
                        })),
                        &_ => Err(mlua::Error::FromLuaConversionError {
                            from: "action",
                            to: String::from("Action"),
                            message: Some(String::from("Invalid file method")),
                        }),
                    }
                }
                "script" => Ok(Self::Script(Script {
                    install: table.get("install")?,
                    update: table.get("update")?,
                    uninstall: table.get("uninstall")?,
                })),
                &_ => Err(mlua::Error::FromLuaConversionError {
                    from: "action",
                    to: String::from("Action"),
                    message: Some(String::from("Invalid action")),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
