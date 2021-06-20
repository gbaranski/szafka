use serde::{de, ser};
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct Szafka<T> {
    path: PathBuf,
    phantom: std::marker::PhantomData<T>,
}

impl<T: de::DeserializeOwned + ser::Serialize> Szafka<T> {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            path: path.as_ref().into(),
            phantom: std::marker::PhantomData::default(),
        }
    }

    /// Overwrite the existing data
    pub async fn save(&self, data: &T) -> Result<(), Error> {
        use tokio::io::AsyncWriteExt;

        let file_contents = serde_json::to_string_pretty(data)?;

        if let Some(path) = self.path.parent() {
            if !path.exists() {
                tokio::fs::create_dir_all(path)
                    .await
                    .map_err(Error::CreateParentError)?;
            }
        }

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)
            .await
            .map_err(Error::OpenFileError)?;

        file.set_len(0_u64)
            .await
            .map_err(Error::ChangeFileLengthError)?;

        file.write_all(file_contents.as_bytes())
            .await
            .map_err(Error::WriteFileError)?;

        Ok(())
    }

    pub async fn get(&self) -> Result<T, Error> {
        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&self.path)
            .await
            .map_err(Error::OpenFileError)?;

        let file: T = serde_json::from_reader(file.into_std().await)?;

        Ok(file)
    }

    pub async fn flush(&self) -> Result<(), Error> {
        if self.path.exists() {
            Ok(tokio::fs::remove_file(&self.path)
                .await
                .map_err(Error::RemoveFileError)?)
        } else {
            Ok(())
        }
    }
}
