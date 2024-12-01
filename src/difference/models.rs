use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Difference {
    pub actions: Vec<Action>,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    File(File),
    Script(Script),
}

#[derive(Debug, PartialEq)]
pub enum File {
    Create { path: PathBuf, content: String },
    Update { path: PathBuf, content: String },
    Delete { path: PathBuf },
}

pub type Script = Vec<String>;
