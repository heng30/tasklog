pub mod def;

#[allow(unused)]
pub use sqldb::{create_db, entry};

pub async fn init(db_path: &str) {
    create_db(db_path).await.expect("create db");

    entry::new(def::RECORD_TABLE)
        .await
        .expect("record table failed");
}
