use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use rand::{RngCore, Rng};
use serde::{Deserialize, Serialize};
use szafka::Szafka;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Something {
    name: String,
    id: u64,
}

impl Something {
    pub fn random() -> Self {

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

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = Runtime::new().expect("create runtime fail");
    c.bench_function("save", |b| {
        b.to_async(&runtime).iter(|| async {
            let szafka = Szafka::new("/tmp/szafka-benches");
            let something = Something::random();
            szafka.save(&something).await.unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
