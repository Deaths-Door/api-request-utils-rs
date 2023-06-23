use std::collections::HashMap;

pub use reqwest::{Client, Error as ReqwestError, RequestBuilder};
pub use serde_json::{Error as SerdeJsonError, Result as SerdeJsonResult};


pub type ParameterHashMap<'a> = HashMap<&'a str, Option<&'a str>>;

/// A trait for handling HTTP requests.
#[async_trait::async_trait]
pub trait RequestHandler<'a> : RequestDefaults<'a> {
    /// The base URL for the requests.
    const BASE_URL : &'static str;

    /// The API key as string used for authentication.
    const API_KEY : Option<&'static str> = Some("apiKey");

    /// Joins the given endpoint with the base URL.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint to join with the base URL.
    ///
    /// # Returns
    ///
    /// The joined URL as a `String`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// struct MyRequestHandler;
    ///
    /// impl<'a> RequestHandler<'a> for MyRequestHandler {
    ///     const BASE_URL: &'static str = "https://api.example.com";
    ///
    ///     fn make_request_url(&self, endpoint: &str) -> String {
    ///         Self::join_endpoints(endpoint)
    ///     }
    /// }
    /// ```
    fn join_endpoints(endpoint : &str) -> String {
        format!("{}/{}",Self::BASE_URL,endpoint)
    }

    /// Builds the parameter hashmap using the given function.
    ///
    /// # Arguments
    ///
    /// * `function` - A closure that takes a mutable reference to a `ParameterHashMap` and modifies it.
    ///
    /// # Returns
    ///
    /// The populated `ParameterHashMap`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// struct MyRequestHandler;
    ///
    /// impl<'a> RequestHandler<'a> for MyRequestHandler {
    ///     const BASE_URL: &'static str = "https://api.example.com";
    ///
    ///     fn make_parameters(&self) -> ParameterHashMap<'a> {
    ///         self.parameters(|params| {
    ///             params.insert("key1", Some("value1"));
    ///             params.insert("key2", Some("value2"));
    ///         })
    ///     }
    /// }
    /// ```
    fn parameters<Function>(&self,function: Function) ->  ParameterHashMap<'a> where Function : FnOnce(&mut ParameterHashMap<'a>) {
        let mut parameters : ParameterHashMap<'a> = HashMap::new();
        function(&mut parameters);
        parameters
    }

    /// Sends an HTTP request with the given `RequestBuilder`, and returns the parsed response.
    ///
    /// # Arguments
    ///
    /// * `request_builder` - The `RequestBuilder` containing the configured request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed response on success, or a `StatusCode` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// struct MyRequestHandler;
    ///
    /// #[derive(serde::Deserialize)]
    /// struct MyResponse {
    ///     // Define your response structure here
    /// }
    ///
    /// #[async_trait::async_trait]
    /// impl<'a> RequestHandler<'a> for MyRequestHandler {
    ///     const BASE_URL: &'static str = "https://api.example.com";
    ///
    ///     async fn make_request<T>(&self, request_builder: RequestBuilder) -> Result<T, StatusCode>
    ///     where
    ///         T: for<'de> serde::Deserialize<'de>,
    ///     {
    ///         self.request(request_builder).await
    ///     }
    /// }
    /// ```
    async fn request<T>(&self,request_builder : reqwest::RequestBuilder) -> std::result::Result<T,reqwest::StatusCode> where T : for<'de> serde::Deserialize<'de> {
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

pub trait RequestDefaults<'a> {
    /// Modifies the provided `RequestBuilder` with default headers.
    ///
    /// # Arguments
    ///
    /// * `request_builder` - The `RequestBuilder` to modify.
    ///
    /// # Returns
    ///
    /// The modified `RequestBuilder` with default headers set.
    fn default_headers(&self,request_builder : reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request_builder
    }

    /// Modifies the provided `RequestBuilder` with default parameters.
    ///
    /// # Arguments
    ///
    /// * `request_builder` - The `RequestBuilder` to modify.
    ///
    /// # Returns
    ///
    /// The modified `RequestBuilder` with default parameters set.
    fn default_parameters(&self,_request_builder : reqwest::RequestBuilder) -> ParameterHashMap {
        HashMap::new()
    }

    /// Modifies the provided `RequestBuilder` with default settings for post request.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint for the request.
    /// * `json` - The JSON payload for the request.
    ///
    /// # Returns
    ///
    /// The modified `RequestBuilder` with default settings applied.
    fn default_post_requestor(&self,endpoint : &str, json : &'a str) -> reqwest::RequestBuilder;

    /// Modifies the provided `RequestBuilder` with default settings for get request.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint for the request.
    /// * `json` - The JSON payload for the request.
    ///
    /// # Returns
    ///
    /// The modified `RequestBuilder` with default settings applied.
    fn default_get_requestor(&self,endpoint : &str,parameters : ParameterHashMap<'a>) -> reqwest::RequestBuilder;
}