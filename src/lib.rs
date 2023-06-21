use std::collections::HashMap;

pub use reqwest::*;

pub type ParameterHashMap<'a> = HashMap<&'a str, Option<&'a str>>;

/// A trait for handling HTTP requests.
#[async_trait::async_trait]
pub trait RequestHandler<'a> {
    /// The base URL for the requests.
    const BASE_URL : &'static str;

    /// The API key as string used for authentication.
    const API_KEY : Option<&'static str> = Some("apiKey");

    /// Builds the parameter hashmap using the given function.
    fn parameters<Function>(&self,function: Function) ->  ParameterHashMap<'a> where Function : FnOnce(&mut ParameterHashMap<'a>) {
        let mut parameters : ParameterHashMap<'a> = HashMap::new();
        function(&mut parameters);
        parameters
    }

    /// Sends an HTTP request with the given endpoint and parameters, and returns the parsed response.
    async fn request<T>(request_builder : reqwest::RequestBuilder) -> std::result::Result<T,reqwest::StatusCode> where T : for<'de> serde::Deserialize<'de> {
        let response = request_builder.send().await.expect("Error requesting request");
        let status = response.status();
        if status.is_success() {
            let body = response.text().await.expect("Error in reading response body");
            let deserialized : T = serde_json::from_str(&body).expect("Error deserializing response body");
            Ok(deserialized)
        }
        else {
            Err(status)
        }
    }
}