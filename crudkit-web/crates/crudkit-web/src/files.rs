// TODO: This should probably be moved to downstream crates!

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum ListFileError {
    S3Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FileResource {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesResponse {
    pub files: Vec<FileResource>,
    pub error: Option<ListFileError>,
}
