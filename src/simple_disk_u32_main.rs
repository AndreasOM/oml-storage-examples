use std::env;
use std::path::Path;
use std::sync::Arc;

use color_eyre::eyre::Result;
use oml_storage::Storage;
use oml_storage::StorageDisk;
use oml_storage::StorageItem;

use serde::Deserialize;
use serde::Serialize;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    color_eyre::install()?;

    tracing::info!("Example started - Simple Disk u32");

    let mut storage: Box<dyn Storage<TestItem>> = {
        let extension = Path::new("test_item");
        let mut path = env::current_dir()?;
        path.push("data");
        path.push("simple_disk_u32");
        path.push("test_items");
        tracing::debug!("Path {path:?} .{extension:?}");

        let storage = StorageDisk::<TestItem>::new(&path, &extension).await;
        Box::new(storage)
    };

    storage.ensure_storage_exists().await?;

    let storage = Arc::new(storage);

    let item_id = TestItem::generate_next_id(None);
    let (lock, mut item) = storage.lock(&item_id, "OWNER_ID").await?.success()?;
    tracing::info!("Item lock {lock:?}");
    item.increment_counter();
    let data = item.data();
    tracing::info!("Data: '{data}'");
    item.set_data("some data");
    storage.save(&item_id, &item, &lock).await?;
    storage.unlock(&item_id, lock).await?;

    tracing::info!("Example ended");
    Ok(())
}

fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TestItem {
    counter: u32,
    #[serde(default)]
    data: String,
}

impl TestItem {
    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn set_data(&mut self, data: &str) {
        self.data = data.to_string();
    }

    fn data(&self) -> &str {
        &self.data
    }
}

impl TestItem {
    fn generate_next_id_u32(
        a_previous_id: Option<&<TestItem as StorageItem>::ID>,
    ) -> <TestItem as StorageItem>::ID {
        tracing::info!("generate_next_id_u32 {a_previous_id:?}");
        let id = if let Some(a_previous_id) = a_previous_id {
            a_previous_id + 1
        } else {
            1
        };
        id
    }
}

impl StorageItem for TestItem {
    type ID = u32;

    fn serialize(&self) -> Result<Vec<u8>> {
        let json = serde_json::to_string_pretty(&self)?;

        Ok(json.into())
    }
    fn deserialize(data: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let i = serde_json::from_slice(&data)?;

        Ok(i)
    }
    fn generate_next_id(a_previous_id: Option<&Self::ID>) -> Self::ID {
        Self::generate_next_id_u32(a_previous_id)
    }
    fn make_id(id: &str) -> Result<Self::ID> {
        let id = id.parse::<Self::ID>()?;
        Ok(id)
    }
}
