use std::path::PathBuf;
use crate::Error;
use serde::ser;
use serde::de;

#[derive(Debug, Clone)]
pub struct Szafka<T> {
    pub path: PathBuf,
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
    /// szafka.save(&something).expect("save failed");
    /// ```
    pub fn save(&self, data: &T) -> Result<(), Error> {
        use std::io::Write;

        let file_contents = serde_json::to_string_pretty(data)?;

        if let Some(path) = self.path.parent() {
            if !path.exists() {
                std::fs::create_dir_all(path)
                    .map_err(Error::CreateParentError)?;
            }
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)
            .map_err(Error::OpenFileError)?;

        file.set_len(0_u64)
            .map_err(Error::ChangeFileLengthError)?;

        file.write_all(file_contents.as_bytes())
            .map_err(Error::WriteFileError)?;

        Ok(())
    }

    /// Retrieve stored data
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// szafka.save(&something).expect("save failed");
    /// let retrieved = szafka.get().expect("get data failed");
    /// assert_eq!(something, retrieved);
    /// ```
    pub fn get(&self) -> Result<T, Error> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(Error::OpenFileError)?;

        let file: T = serde_json::from_reader(file)?;

        Ok(file)
    }

    /// Check if there is any saved data
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// szafka.save(&something).expect("save failed");
    /// assert!(szafka.exists());
    /// ```
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Remove stored data
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// szafka.save(&something).expect("save failed");
    /// szafka.remove().expect("remove failed");
    /// assert!(szafka.exists() == false);
    /// ```
    pub fn remove(&self) -> Result<(), Error> {
        if self.path.exists() {
            Ok(std::fs::remove_file(&self.path)
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

    fn get_szafka<T: de::DeserializeOwned + ser::Serialize>() -> Szafka<T> {
        let szafka = Szafka::new(format!("/tmp/szafka-test-{}", id()));
        szafka.remove().unwrap();
        szafka
    }

    fn teardown<T: de::DeserializeOwned + ser::Serialize>(szafka: Szafka<T>) {
        szafka.remove().unwrap()
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

    #[test]
    fn no_parent() {
        let szafka = Szafka::new("/tmp/szafka/subdir/file");
        let something = Something::random();
        szafka.save(&something).unwrap();
        assert!(szafka.exists());
        assert_eq!(szafka.get().unwrap(), something);

        teardown(szafka)
    }


    #[test]
    fn get() {
        let szafka = get_szafka::<Something>();
        let something = Something::random();
        szafka.save(&something).unwrap();
        assert!(szafka.exists());
        assert_eq!(szafka.get().unwrap(), something);

        teardown(szafka)
    }

    #[test]
    fn remove() {
        let szafka = get_szafka::<Something>();
        let something = Something::random();
        szafka.save(&something).unwrap();
        assert!(szafka.exists());
        szafka.remove().unwrap();
        szafka.remove().unwrap();
        assert!(!szafka.exists());
        teardown(szafka)
    }

    #[test]
    fn exists() {
        let szafka = get_szafka::<Something>();
        let something = Something::random();
        szafka.save(&something).unwrap();
        assert!(szafka.exists());

        teardown(szafka)
    }
}