use crate::generations;

use self::models::Difference;

pub mod models;
#[cfg(test)]
mod tests;

pub fn differ_generations(
    initial_generation: &generations::models::Generation,
    final_generation: &generations::models::Generation,
) -> Difference {
    let mut actions: Vec<models::Action> = Vec::new();

    let mut file_actions = differ_generation_files(initial_generation, final_generation);
    actions.append(&mut file_actions);

    let mut script_actions = differ_generation_scripts(initial_generation, final_generation);
    actions.append(&mut script_actions);

    Difference { actions }
}

fn differ_generation_files(
    initial_generation: &generations::models::Generation,
    final_generation: &generations::models::Generation,
) -> Vec<models::Action> {
    let mut actions: Vec<models::Action> = Vec::new();

    for final_file in &final_generation.files {
        let mut path_matched = false;

        for initial_file in &initial_generation.files {
            if initial_file.path != final_file.path {
                continue;
            }

            path_matched = true;

            if initial_file.content == final_file.content {
                break;
            }

            if let Some(content) = &final_file.content {
                actions.push(models::Action::File(models::File::Update {
                    path: final_file.path.clone(),
                    content: content.to_string(),
                }))
            } else {
                actions.push(models::Action::File(models::File::Delete {
                    path: final_file.path.clone(),
                }))
            }

            break;
        }

        if path_matched {
            continue;
        }

        if let Some(content) = &final_file.content {
            actions.push(models::Action::File(models::File::Create {
                path: final_file.path.clone(),
                content: content.to_string(),
            }))
        } else {
            actions.push(models::Action::File(models::File::Delete {
                path: final_file.path.clone(),
            }))
        }
    }

    for initial_file in &initial_generation.files {
        let mut path_matched = false;

        for final_file in &final_generation.files {
            if final_file.path == initial_file.path {
                path_matched = true;
                break;
            }
        }

        if path_matched {
            continue;
        }

        actions.push(models::Action::File(models::File::Delete {
            path: initial_file.path.clone(),
        }))
    }

    actions
}

fn differ_generation_scripts(
    initial_generation: &generations::models::Generation,
    final_generation: &generations::models::Generation,
) -> Vec<models::Action> {
    let mut actions: Vec<models::Action> = Vec::new();

    for initial_script in &initial_generation.scripts {
        let mut install_matched = false;

        for final_script in &final_generation.scripts {
            if final_script.install == initial_script.install {
                install_matched = true;
                break;
            }
        }

        if install_matched {
            continue;
        }

        actions.push(models::Action::Script(initial_script.uninstall.clone()))
    }

    for final_script in &final_generation.scripts {
        let mut install_matched = false;

        for initial_script in &initial_generation.scripts {
            if initial_script.install == final_script.install {
                install_matched = true;
                break;
            }
        }

        if install_matched {
            continue;
        }

        actions.push(models::Action::Script(final_script.install.clone()))
    }

    actions
}
