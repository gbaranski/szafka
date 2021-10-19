use serde::{Deserialize, Serialize};
use szafka::Szafka;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Something {
    name: String,
    id: u64,
}

fn main() {
    let szafka = Szafka::new("/tmp/welcome-to-szafka");
    let something = Something {
        name: String::from("John"),
        id: 1000,
    };
    szafka.save(&something).expect("save failed");
    let retrieved = szafka.get().expect("get failed");
    assert_eq!(something, retrieved);
    assert!(szafka.exists());
    szafka.remove().expect("remove failed");
    assert!(!szafka.exists());
}
