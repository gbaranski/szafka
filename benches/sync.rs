use criterion::{criterion_group, criterion_main, Criterion};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use szafka::Szafka;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Something {
    name: String,
    id: u64,
}

impl Something {
    #[cfg(not(tarpaulin_include))]
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
    let szafka = Szafka::new("/tmp/szafka-benches");

    c.bench_function("save", |b| {
        b.iter(|| {
            let something = Something::random();
            szafka.save(&something).unwrap();
        })
    });

    let something = Something::random();
    szafka.save(&something).unwrap();
    c.bench_function("get", |b| {
        b.iter(|| {
            assert_eq!(szafka.get().unwrap(), something);
        })
    });
    szafka.remove().unwrap();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
