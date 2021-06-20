# szafka
Asynchronous persistent data store in Rust.

## Usage

Basic usage:

```rust
#[tokio::main]
async fn main() {
    use szafka::Szafka;
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Something {
        name: String,
        id: u64,
    }
    
    let szafka = Szafka::new("/tmp/welcome-to-szafka");
    let something = Something {
        name: String::from("John"),
        id: 1000,
    };
    szafka.save(&something).await.expect("save failed");
    let retrieved = szafka.get().await.expect("get data failed");
    assert_eq!(something, retrieved);
}
```

More information can be found in [the documentation](https://docs.rs/szafka)

## Why

I needed to store persistant data such as bearer tokens or cached data somewhere for [Houseflow CLI App](https://github.com/gbaranski/houseflow)
