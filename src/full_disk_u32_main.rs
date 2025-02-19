use crate::full_example::FullExample;
use crate::test_item::TestItem;
use std::env;
use std::path::Path;
use std::sync::Arc;

use color_eyre::eyre::Result;
use oml_storage::Storage;
use oml_storage::StorageDisk;

mod full_example;
mod test_item;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    color_eyre::install()?;

    let random_folder_name = nanoid::nanoid!();

    tracing::info!("Example started - Full Disk u32 - {random_folder_name}");

    let mut storage: Box<dyn Storage<TestItem>> = {
        let extension = Path::new("test_item");
        let mut path = env::current_dir()?;
        path.push("data");
        path.push("full_disk_u32");
        // path.push(random_folder_name);
        path.push("test_items");
        tracing::debug!("Path {path:?} .{extension:?}");

        let storage = StorageDisk::<TestItem>::new(&path, extension).await;
        Box::new(storage)
    };

    storage.ensure_storage_exists().await?;

    let storage = Arc::new(storage);

    let mut example = FullExample::default();
    // example.set_delay_in_seconds( 15 ); // enable to observe
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
