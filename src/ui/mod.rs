pub mod hands;
pub mod requester;
pub use hands::HandBuilderGUI;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("An error has occur: {0}")]
    ImpossibleState(String),
}
