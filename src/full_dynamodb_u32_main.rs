use crate::full_example::FullExample;
use crate::test_item::TestItem;
use color_eyre::eyre::Result;
use oml_storage::Storage;
use oml_storage::StorageDynamoDb;
use std::sync::Arc;

mod full_example;
mod test_item;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    color_eyre::install()?;

    let random_name = nanoid::nanoid!();

    tracing::info!("Example started - Full DynamoDb u32 - {random_name}");

    let mut storage: Box<dyn Storage<TestItem>> = {
        //let storage = StorageDynamoDb::<TestItem>::new(&random_name).await;
        let table_name = "test_items";
        let mut storage = StorageDynamoDb::<TestItem>::new(&table_name).await;
        storage.set_endpoint_url("http://localhost:8000")?;

        Box::new(storage)
    };

    storage.ensure_storage_exists().await?;

    let storage = Arc::new(storage);

    let mut example = FullExample::default();
    example.run(storage.clone()).await?;

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
