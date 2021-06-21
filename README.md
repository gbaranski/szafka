![Crates.io](https://img.shields.io/crates/d/szafka)
![Crates.io](https://img.shields.io/crates/v/szafka)
![docs.rs](https://img.shields.io/docsrs/szafka)

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

## Tests

All tests: `cargo test`.
Documentation tests: `cargo test --doc`.
Unit tests: `cargo test --tests`.

Check code coverage using [tarpaulin](https://github.com/xd009642/tarpaulin).

```rust
cargo tarpaulin --lib
Jun 21 02:05:24.123  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src/lib.rs: 70/79 +0%
||
88.61% coverage, 70/79 lines covered, +0% change in coverage
```

## Benchmarks

Run benchmarks using `cargo bench`.

Results from my Dell XPS 7390 machine with Intel i7-10510U, NVMe, Linux 5.10

```
save                    time:   [24.651 us 24.757 us 24.861 us]
get                     time:   [34.889 us 35.863 us 36.809 us]
```

## Why

I needed to store persistant data such as bearer tokens or cached data somewhere for [Houseflow CLI App](https://github.com/gbaranski/houseflow)
