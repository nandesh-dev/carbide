use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mlua::{Lua, LuaOptions, Result, StdLib, Table};

pub fn load(directory: PathBuf) -> Result<Vec<models::Action>> {
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
    Ok(actions)
}

pub mod models {
    use std::path::PathBuf;

    use mlua::{FromLua, IntoLua, Lua, Value};

    #[derive(Debug, Clone)]
    pub enum Action {
        Script(Script),
        File(File),
    }

    #[derive(Debug, Clone)]
    pub struct Script {
        pub install: Vec<String>,
        pub update: Vec<String>,
        pub uninstall: Vec<String>,
    }

    #[derive(Debug, Clone)]
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
