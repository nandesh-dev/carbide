use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mlua::{Lua, LuaOptions, Result, StdLib, Table};

pub mod models;
#[cfg(test)]
mod tests;

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
