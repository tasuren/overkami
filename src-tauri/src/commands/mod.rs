pub mod config;
pub mod os;
pub mod sync;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CommandError {
    code: String,
    detail: Option<String>,
}
