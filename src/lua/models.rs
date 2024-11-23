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
