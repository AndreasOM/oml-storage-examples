use crate::TestItem;
use color_eyre::Result;
use oml_storage::StorageItem;
use std::sync::Arc;

use oml_storage::Storage;

#[derive(Debug, Default)]
pub struct SimpleExample {}

impl SimpleExample {
    pub async fn run(&mut self, storage: Arc<Box<dyn Storage<TestItem>>>) -> Result<()> {
        let item_id = TestItem::generate_next_id(None);
        let (lock, mut item) = storage.lock(&item_id, "OWNER_ID").await?.success()?;
        tracing::info!("Item lock {lock:?}");
        item.increment_counter();
        let data = item.data();
        tracing::info!("Data: '{data}'");
        item.set_data("some data");
        storage.save(&item_id, &item, &lock).await?;
        storage.unlock(&item_id, lock).await?;

        Ok(())
    }
}
