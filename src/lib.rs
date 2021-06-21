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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use szafka::Szafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-szafka");
    /// let something = Something {
    ///     name: String::from("John"),
    ///     id: 1000,
    /// };
    /// szafka.save(&something).await.expect("save failed");
    /// # })
    /// ```
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

    /// Retrieve stored data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use szafka::Szafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-szafka");
    /// let something = Something {
    ///     name: String::from("John"),
    ///     id: 1000,
    /// };
    /// szafka.save(&something).await.expect("save failed");
    /// let retrieved = szafka.get().await.expect("get data failed");
    /// assert_eq!(something, retrieved);
    /// # })
    /// ```
    pub async fn get(&self) -> Result<T, Error> {
        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&self.path)
            .await
            .map_err(Error::OpenFileError)?;

        let file: T = serde_json::from_reader(file.into_std().await)?;

        Ok(file)
    }

    /// Check if there is any saved data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use szafka::Szafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-szafka");
    /// let something = Something {
    ///     name: String::from("John"),
    ///     id: 1000,
    /// };
    /// szafka.save(&something).await.expect("save failed");
    /// assert!(szafka.exists());
    /// # })
    /// ```
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Flush stored data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use szafka::Szafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-szafka");
    /// let something = Something {
    ///     name: String::from("John"),
    ///     id: 1000,
    /// };
    /// szafka.save(&something).await.expect("save failed");
    /// szafka.flush().await.expect("flush failed");
    /// assert!(szafka.exists() == false);
    /// # })
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    fn id() -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static ID: AtomicUsize = AtomicUsize::new(0);
        ID.fetch_add(1, Ordering::SeqCst)
    }

    async fn get_szafka<T: de::DeserializeOwned + ser::Serialize>() -> Szafka<T> {
        let szafka = Szafka::new(format!("/tmp/szafka-test-{}", id()));
        szafka.flush().await.unwrap();
        szafka
    }

    async fn teardown<T: de::DeserializeOwned + ser::Serialize>(szafka: Szafka<T>) {
        szafka.flush().await.unwrap()
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Something {
        name: String,
        id: u64,
    }

    impl Something {
        pub fn random() -> Self {
            use rand::{Rng, RngCore};

            let mut rng = rand::thread_rng();
            let mut name = [0; 16];
            rng.fill_bytes(&mut name);
            let name = hex::encode(name);
            Self {
                name,
                id: rng.gen(),
            }
        }
    }

    #[tokio::test]
    async fn save_no_parent() {
        let szafka = Szafka::new("/tmp/szafka/subdir/file");
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());
        assert_eq!(szafka.get().await.unwrap(), something);

        teardown(szafka).await
    }


    #[tokio::test]
    async fn save_get() {
        let szafka = get_szafka::<Something>().await;
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());
        assert_eq!(szafka.get().await.unwrap(), something);

        teardown(szafka).await
    }

    #[tokio::test]
    async fn save_flush() {
        let szafka = get_szafka::<Something>().await;
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());
        szafka.flush().await.unwrap();
        szafka.flush().await.unwrap();
        assert!(!szafka.exists());
        teardown(szafka).await
    }

    #[tokio::test]
    async fn save_exists() {
        let szafka = get_szafka::<Something>().await;
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());

        teardown(szafka).await
    }
}
