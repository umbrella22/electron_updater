use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FileHashAndPath {
    pub filePath: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct UpdateConfigJson {
    pub added: Vec<FileHashAndPath>,
    pub changed: Vec<FileHashAndPath>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RunningState {
    Nothing = 0,
    Updating,
    UpdateButNotCheck,
    Finish,
    Failed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunningConfig {
    pub status: RunningState,
    pub file_path: HashMap<usize, String>,
    pub update_temp_path: String,
    pub exe_path: String,
    pub moved_path: Vec<String>,
}
