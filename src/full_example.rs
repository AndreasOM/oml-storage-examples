use crate::TestItem;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use oml_storage::LockResult;
use oml_storage::StorageItem;
use std::sync::Arc;

use oml_storage::Storage;

#[derive(Debug, Default)]
pub struct FullExample {
    delay: u64,
}

impl FullExample {
    pub fn set_delay_in_seconds(&mut self, delay: u64) {
        self.delay = delay;
    }
    pub async fn run(&mut self, storage: Arc<Box<dyn Storage<TestItem>>>) -> Result<()> {
        let delay = std::time::Duration::from_secs(self.delay);

        // Notes:
        // - We use the same storage for different scenarios, in real world you'd split into multiple storages

        let us = "example_node_id_of_us";

        // --= Scenario: Login/Signup =--
        // user signs up with external ID, e.g. device identifier, or email, or....
        let external_id = 42;
        let external_secret = "forty_two";
        let (lock, mut item) = match storage.lock(&external_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { who } => {
                tracing::warn!("{external_id} should not be locked by {who} - aborting example");
                return Err(eyre!("example failed"));
            }
        };
        // initialise if new user, otherwise authenticate
        if let Some(secret) = item.secret() {
            // existing login, hash, and verify
            // -> done
            tracing::info!("Existing item for {external_id} -> {secret}");
        // could load account data, but lets keep things linear
        } else {
            // new login, hash, and store
            tracing::info!("New item for {external_id} <- {external_secret}");
            item.set_secret(&external_secret);
        }
        storage.save(&external_id, &item, &lock).await?;
        storage.unlock(&external_id, lock).await?;
        // send reponse to caller

        // --= Scenario: User returns to modify their data =--
        //
        let external_id = 42;
        let (lock, mut item) = match storage.lock(&external_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { who } => {
                tracing::warn!("{external_id} should not be locked by {who} - aborting example");
                return Err(eyre!("example failed"));
            }
        };

        // modify something
        tracing::info!("Counter for {external_id}: {}", item.counter());
        item.increment_counter();
        storage.save(&external_id, &item, &lock).await?;
        storage.unlock(&external_id, lock).await?;
        // send reponse to caller

        // --= Scenario: User returns to modify their data from two different browsers =--
        //
        let external_id = 42;
        let (lock, mut item) = match storage.lock(&external_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { who } => {
                tracing::warn!("{external_id} should not be locked by {who} - aborting example");
                return Err(eyre!("example failed"));
            }
        };

        // modify something
        tracing::info!("Counter for {external_id}: {}", item.counter());
        item.increment_counter();

        // while we are working, the second request arrives
        match storage.lock(&external_id, &us).await? {
            LockResult::Success { lock: _, item: _ } => {
                tracing::warn!("{external_id} should be locked - aborting example");
                return Err(eyre!("example failed"));
            }
            LockResult::AlreadyLocked { who } => {
                // :TODO: double check who
                tracing::info!("{external_id} already locked as expected by {who}");
            }
        };

        storage.save(&external_id, &item, &lock).await?;
        storage.unlock(&external_id, lock).await?;
        // send reponse to caller

        // second caller tries again later
        let external_id = 42;
        let (lock, mut item) = match storage.lock(&external_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { who } => {
                tracing::warn!("{external_id} should not be locked by {who} - aborting example");
                return Err(eyre!("example failed"));
            }
        };

        // modify something
        tracing::info!("Counter for {external_id}: {}", item.counter());
        item.increment_counter();

        storage.save(&external_id, &item, &lock).await?;
        storage.unlock(&external_id, lock).await?;

        // --= Scenario: Admin cleans up after crash, and force unlocks "stale" locks =--

        let external_id = 42;
        let (lock, item) = match storage.lock(&external_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { who } => {
                tracing::warn!("{external_id} should not be locked by {who} - aborting example");
                return Err(eyre!("example failed"));
            }
        };
        // "crash"
        drop(lock);
        drop(item);
        std::thread::sleep(delay);

        // admin initiates force unlock
        storage.force_unlock(&external_id).await?;
        std::thread::sleep(delay);

        // user comes back, everything fine
        let external_id = 42;
        let (lock, _item) = match storage.lock(&external_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { who } => {
                tracing::warn!("{external_id} should not be locked by {who} - aborting example");
                return Err(eyre!("example failed"));
            }
        };
        storage.unlock(&external_id, lock).await?;

        // --= Scenario: Autogenerate "random" IDs - no collision =--

        let item_id = TestItem::generate_next_id(None);
        let (lock, mut item) = storage.lock(&item_id, &us).await?.success()?;
        item.increment_counter();
        let data = item.data();
        tracing::info!("Data: '{data}'");
        item.set_data("some data");
        storage.save(&item_id, &item, &lock).await?;
        storage.unlock(&item_id, lock).await?;

        // --= Scenario: Autogenerate "random" IDs - collision =--
        // :TODO: these scenarios are actually more complex

        let item_id = TestItem::generate_next_id(Some(&1001));
        if storage.exists(&item_id).await? {
            tracing::warn!("{item_id} should not exist - aborting example");
            return Err(eyre!("example failed"));
        }
        let (lock, mut item) = storage.lock(&item_id, &us).await?.success()?;
        item.increment_counter();
        let data = item.data();
        tracing::info!("Data: '{data}'");
        item.set_data("some data");
        storage.save(&item_id, &item, &lock).await?;
        storage.unlock(&item_id, lock).await?;

        // force collision
        let item_id = TestItem::generate_next_id(Some(&1001));
        if !storage.exists(&item_id).await? {
            tracing::warn!("{item_id} should exist - aborting example");
            return Err(eyre!("example failed"));
        }
        let item_id = TestItem::generate_next_id(Some(&item_id));
        if storage.exists(&item_id).await? {
            tracing::warn!("{item_id} should not exsit - aborting example");
            return Err(eyre!("example failed"));
        }
        let (lock, mut item) = storage.lock(&item_id, &us).await?.success()?;
        item.increment_counter();
        let data = item.data();
        tracing::info!("Data: '{data}'");
        item.set_data("some data");
        storage.save(&item_id, &item, &lock).await?;
        storage.unlock(&item_id, lock).await?;

        Ok(())
    }
}
