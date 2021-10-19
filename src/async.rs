use std::path::PathBuf;
use crate::Error;
use serde::ser;
use serde::de;

#[derive(Debug, Clone)]
pub struct AsyncSzafka<T> {
    pub path: PathBuf,
    phantom: std::marker::PhantomData<T>,
}

impl<T: de::DeserializeOwned + ser::Serialize> AsyncSzafka<T> {
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
    /// use szafka::AsyncSzafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-async-szafka");
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
    /// use szafka::AsyncSzafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-async-szafka");
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
    /// use szafka::AsyncSzafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-async-szafka");
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

    /// Remove stored data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use szafka::AsyncSzafka;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    /// struct Something {
    ///     name: String,
    ///     id: u64,
    /// }
    ///
    /// let szafka = Szafka::new("/tmp/welcome-to-async-szafka");
    /// let something = Something {
    ///     name: String::from("John"),
    ///     id: 1000,
    /// };
    /// szafka.save(&something).await.expect("save failed");
    /// szafka.remove().await.expect("remove failed");
    /// assert!(szafka.exists() == false);
    /// # })
    /// ```
    pub async fn remove(&self) -> Result<(), Error> {
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

    async fn get_szafka<T: de::DeserializeOwned + ser::Serialize>() -> AsyncSzafka<T> {
        let szafka = AsyncSzafka::new(format!("/tmp/async-szafka-test-{}", id()));
        szafka.remove().await.unwrap();
        szafka
    }

    async fn teardown<T: de::DeserializeOwned + ser::Serialize>(szafka: AsyncSzafka<T>) {
        szafka.remove().await.unwrap()
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
    async fn no_parent() {
        let szafka = AsyncSzafka::new("/tmp/async-szafka/subdir/file");
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());
        assert_eq!(szafka.get().await.unwrap(), something);

        teardown(szafka).await
    }


    #[tokio::test]
    async fn get() {
        let szafka = get_szafka::<Something>().await;
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());
        assert_eq!(szafka.get().await.unwrap(), something);

        teardown(szafka).await
    }

    #[tokio::test]
    async fn remove() {
        let szafka = get_szafka::<Something>().await;
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());
        szafka.remove().await.unwrap();
        szafka.remove().await.unwrap();
        assert!(!szafka.exists());
        teardown(szafka).await
    }

    #[tokio::test]
    async fn exists() {
        let szafka = get_szafka::<Something>().await;
        let something = Something::random();
        szafka.save(&something).await.unwrap();
        assert!(szafka.exists());

        teardown(szafka).await
    }
}