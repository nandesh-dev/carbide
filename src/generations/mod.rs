use regex::Regex;
use std::{fs, io, path::PathBuf};

pub mod models;
#[cfg(test)]
mod tests;

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

            generations.push(models::Generation::from_file(&path)?)
        }
    }

    generations.sort_by_key(|g| g.id);

    Ok(generations)
}

pub fn read_last_generation(path: &PathBuf) -> io::Result<models::Generation> {
    let mut highest_id = -1;

    for entry in fs::read_dir(path)? {
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
        }
    }

    if highest_id == -1 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No generation files found",
        ));
    }

    Ok(models::Generation::from_file(
        &path.join(PathBuf::from(format!("carbide-{}", highest_id))),
    )?)
}
