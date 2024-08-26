pub mod brawlify;
pub mod images;
pub mod official_brawl_stars;

use crate::BotError;
use anyhow::anyhow;
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;

use self::{brawlify::BrawlifyAPI, images::ImagesAPI, official_brawl_stars::BrawlStarsAPI};

/// Contains the APIs the bot retrieves resources from, including third-party and local APIs.
#[derive(Debug)]
pub struct APIsContainer {
    pub brawl_stars: BrawlStarsAPI,
    pub brawlify: BrawlifyAPI,
    pub images: ImagesAPI,
}

impl APIsContainer {
    pub fn new() -> Self {
        Self {
            brawl_stars: BrawlStarsAPI::new(),
            brawlify: BrawlifyAPI::new(),
            images: ImagesAPI::new(),
        }
    }
}

/// Wrapper for the result of an API call.
#[derive(Debug)]
pub enum APIResult<M> {
    Ok(M),
    NotFound,
    Maintenance,
}

impl<M> APIResult<M>
where
    M: DeserializeOwned,
{
    /// Create an API result from a response.
    ///
    /// If the response code is 200, an Ok variant will be returned containing the json data, which
    /// can then be deserialized into any type that implements Serialize.
    ///
    /// Errors if the response code is something that is either not covered by the API
    /// documentation or is not something that can be appropriately dealt with by the bot.
    pub async fn from_response(response: Response) -> Result<Self, BotError> {
        match response.status() {
            StatusCode::OK => Ok(APIResult::Ok(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(APIResult::NotFound),
            StatusCode::SERVICE_UNAVAILABLE => Ok(APIResult::Maintenance),
            _ => Err(anyhow!(
                "Request failed with status code: {}\n\nResponse details: {:#?}",
                response.status(),
                response
            )),
        }
    }
}

/// Convenience type to store the url of an API endpoint and append to it.
#[derive(Debug)]
pub struct Endpoint {
    url: String,
}

impl Endpoint {
    fn new(url: String) -> Self {
        Self { url }
    }
    /// Append a path to retrieve a specific resource from the endpoint. e.g. pass in
    /// format!("players/%23{}", player_tag) to get a specific player profile.
    ///
    /// Refer to the API documentation for the exact path.
    fn append_path(&self, path: &str) -> String {
        let mut full_url = self.url.clone();

        full_url.push_str(path);

        full_url
    }
}
