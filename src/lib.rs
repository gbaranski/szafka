#[cfg(feature = "sync")]
mod sync;
#[cfg(feature = "sync")]
pub use sync::Szafka;

#[cfg(any(feature = "async", test))]
mod r#async;
#[cfg(any(feature = "async", test))]
pub use r#async::AsyncSzafka;



#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("serialize data error: `{0}`")]
    SerdeError(#[from] serde_json::Error),
    #[error("create parent error: `{0}`")]
    CreateParentError(std::io::Error),
    #[error("open file error: `{0}`")]
    OpenFileError(std::io::Error),
    #[error("write file error: `{0}`")]
    WriteFileError(std::io::Error),
    #[error("remove file error: `{0}`")]
    RemoveFileError(std::io::Error),
    #[error("change file length error: `{0}`")]
    ChangeFileLengthError(std::io::Error),
}
