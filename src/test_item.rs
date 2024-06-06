use color_eyre::Result;
use oml_storage::StorageItem;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TestItem {
    counter: u32,
    #[serde(default)]
    data: String,
    #[serde(default)]
    secret: Option<String>,
}

impl TestItem {
    pub fn increment_counter(&mut self) {
        self.counter += 1;
    }
    pub fn counter(&self) -> u32 {
        self.counter
    }

    pub fn set_data(&mut self, data: &str) {
        self.data = data.to_string();
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn secret(&self) -> Option<&str> {
        self.secret.as_deref()
    }
    pub fn set_secret<S>(&mut self, secret: S)
    where
        S: ToString,
    {
        self.secret = Some(secret.to_string());
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
