![Crates.io](https://img.shields.io/crates/d/szafka)
![Crates.io](https://img.shields.io/crates/v/szafka)
![docs.rs](https://img.shields.io/docsrs/szafka)

# szafka
Persistent data store in Rust.


## Usage

Sync API usage:

```rust
fn main() {
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
    szafka.save(&something).expect("save failed");
    let retrieved = szafka.get().expect("get data failed");
    assert_eq!(something, retrieved);
}
```


Async API usage(enable with `async` feature):

```rust
#[tokio::main]
async fn main() {
    use szafka::AsyncSzafka;
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Something {
        name: String,
        id: u64,
    }
    
    let szafka = AsyncSzafka::new("/tmp/welcome-to-async-szafka");
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

## Tests

All tests: `cargo test`.
Documentation tests: `cargo test --doc`.
Unit tests: `cargo test --tests`.

Check code coverage using [tarpaulin](https://github.com/xd009642/tarpaulin).

```rust
cargo tarpaulin --lib
Oct 19 22:04:02.086  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/async.rs: 70/79
|| src/sync.rs: 68/72
|| 
91.39% coverage, 138/151 lines covered
```

## Benchmarks

Run benchmarks using `cargo bench`.

Results from my Optiplex 9020 machine with Intel i7-4790, NVMe, Linux 5.14

```
save                    time:   [7.3328 us 7.4197 us 7.5088 us]
get                     time:   [48.460 us 48.577 us 48.737 us]

async-save              time:   [22.558 us 22.745 us 22.930 us]
async-get               time:   [58.201 us 58.599 us 59.142 us]
```

## Why

I needed to store persistant data such as bearer tokens or cached data somewhere for [Houseflow CLI App](https://github.com/gbaranski/houseflow)
