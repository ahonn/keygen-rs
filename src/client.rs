use crate::errors::Error;
use reqwest::Client as ReqwestClient;

pub struct Client {
    inner: ReqwestClient,
    base_url: String,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        unimplemented!()
    }

    pub async fn get<T>(&self, path: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        unimplemented!()
    }

    pub async fn post<T>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        unimplemented!()
    }

    // Implement other HTTP methods as needed
}
