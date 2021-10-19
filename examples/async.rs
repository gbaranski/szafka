use serde::{Deserialize, Serialize};
use szafka::AsyncSzafka;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Something {
    name: String,
    id: u64,
}

#[tokio::main]
async fn main() {
    let szafka = AsyncSzafka::new("/tmp/welcome-to-async-szafka");
    let something = Something {
        name: String::from("John"),
        id: 1000,
    };
    szafka.save(&something).await.expect("save failed");
    let retrieved = szafka.get().await.expect("get failed");
    assert_eq!(something, retrieved);
    assert!(szafka.exists());
    szafka.remove().await.expect("remove failed");
    assert!(!szafka.exists());
}
