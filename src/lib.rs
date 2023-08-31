#![doc = include_str!("../README.md")]
#![forbid(missing_docs)]

use std::collections::HashMap;

use reqwest::{
    Client,
    RequestBuilder,
}; 

use serde_json::Value;
use serde::de::DeserializeOwned;

use async_trait::async_trait;

use thiserror::Error as ErrorMacro;

pub use reqwest;
pub use serde_json;
pub use serde;
pub use ::async_trait;

/// Trait to provide some basic info about API
pub trait RequestInfo {
    /// The base URL for the requests.
    const BASE_URL : &'static str;

    /// Returns the [reqwest::Client] instance associated with the API client.
    ///
    /// The client is used to send HTTP requests to the API.
    fn client(&self) -> &Client;
}

/// This trait provides methods for modifying the struct in a specific way:
pub trait RequestModifiers: RequestInfo  {
    /// Joins the given endpoint with the base URL.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint to join with the base URL.
    ///
    /// # Returns
    ///
    /// The joined URL as a `String`.
    fn create_endpoint(endpoint : &str) -> String {
        format!("{}/{}",Self::BASE_URL,endpoint)
    }

    /// Conditionally adds a header to the given `RequestBuilder` based on the result of a closure.
    ///
    /// If the closure returns `true`, the specified header with the provided `key` and `value`
    /// will be added to the request. If the closure returns `false`, no changes will be made
    /// to the request.
    ///
    /// # Arguments
    ///
    /// * `request_builder` - The `RequestBuilder` to add the header to.
    /// * `key` - The key of the header to be added.
    /// * `value` - The value of the header to be added.
    /// * `closure` - A closure that determines whether the header should be added. It should
    ///               take no arguments and return a boolean value.
    ///
    /// # Returns
    ///
    /// The modified `RequestBuilder` with the header added if the closure returns `true`,
    /// otherwise the original `RequestBuilder` without any modifications.
    fn add_header_if(request_builder: RequestBuilder,key: &str, value: &str,closure : impl FnOnce() -> bool) -> RequestBuilder {
        match closure() {
            true => request_builder.header(key, value),
            false => request_builder
        }
    }
}

/// The RequestDefaults trait provides default methods for configuring and modifying HTTP requests.
pub trait RequestDefaults: RequestModifiers {
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
    fn default_parameters(&self,request_builder : reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request_builder
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
    fn default_post_requestor(&self,endpoint : &str, json : String) -> reqwest::RequestBuilder {
        self.default_parameters(self.default_headers(self.client().post(Self::create_endpoint(endpoint)))).body(json)
    }

    /// Modifies the provided `RequestBuilder` with default settings for get request.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint for the request.
    /// * `parameters` - The Parameters for the request.
    ///
    /// # Returns
    ///
    /// The modified `RequestBuilder` with default settings applied.
    fn default_get_requestor<'a>(&self,endpoint : &str,parameters : &HashMap<&'a str,Value>) -> reqwest::RequestBuilder {
        self.default_parameters(self.default_headers(self.client().get(Self::create_endpoint(endpoint)))).query(&parameters)
    }
}


/// A trait for handling HTTP requests.
#[async_trait]
pub trait RequestHandler<T : DeserializeOwned,O : DeserializeOwned,E : DeserializeOwned> : RequestDefaults {
    /// Sends an HTTP request, processes the response, and maps it using the provided closure.
    ///
    /// This asynchronous function sends an HTTP request using the given `reqwest::RequestBuilder`,
    /// processes the response, and maps it using the provided closure. It returns the mapped
    /// result if the request is successful, or an `RequestError::ErrorPayload` variant if the
    /// request fails.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the struct implementing this trait.
    /// * `request` - The `reqwest::RequestBuilder` representing the request to be sent.
    /// * `map` - A closure that maps the successful response JSON into the desired output type. Just write `|x| x` if the not mapping is required. 
    ///
    /// # Returns
    ///
    /// A `Result` containing the mapped output type or an `RequestError` variant.
    async fn request_map(request: reqwest::RequestBuilder,map : impl FnOnce(T) -> O + Send + Sync) -> Result<O,RequestError<E>> {
        let response = request.send().await?;
        let status = response.status();

        let body = response.bytes().await?;
        
        match status.is_success() {
            true => {
                let json = serde_json::from_slice(&body)?;
                Ok(map(json))
            }
            false => {
                let json = serde_json::from_slice(&body)?;
                Err(RequestError::ErrorPayload(json))
            }
        }
    }

    /// Resolves the error in the response and returns an option containing the value or `None`.
    ///
    /// # Arguments
    ///
    /// * `response` - The response as a `Result` type.
    /// * `error_resolver` - The closure that handles the error and performs custom error handling.
    ///
    /// # Returns
    ///
    /// An option containing the value if the response is successful, otherwise `None`.
    fn resolve_error(&self,response : Result<O,RequestError<E>>,error_handler : impl Fn(RequestError<E>) + Sync) -> Option<O> {
        match response {
            Ok(value) => Some(value),
            Err(error) => {
                error_handler(error);
                None
            }
        }
    }

    /// This asynchronous function constructs (by default) a GET request using the `default_get_requestor` method
    /// with the given endpoint and parameters. It then sends the request using the request method, expecting
    /// a response of type `T` or an error of type `E`. The error is resolved using the `resolve_error` method
    /// and returns an `Option<T>` representing the response data if successful, or `None` if an error occurred.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint URL to send the GET request to.
    /// * `parameters` - A hashmap containing any parameters to include in the request.
    /// * `map` - A closure that maps the successful response JSON into the desired output type.
    /// * `error_handler` - A closure that handles the error case if an `RequestError` occurs.
    ///
    /// # Returns
    ///
    /// An `Option<O>` representing the response data if successful, or `None` if an error occurred.
    async fn get_request_handler<'a>(&self,endpoint : &str,parameters : &HashMap<&'a str,Value>,map : impl FnOnce(T) -> O + Send + Sync,error_handler : impl Fn(RequestError<E>) + Sync + Send) -> Option<O> { 
        let request = self.default_get_requestor(endpoint,parameters);
        let response = Self::request_map(request,map).await;
        self.resolve_error(response,error_handler)
    }

    /// Handles a POST request to the specified endpoint with the provided JSON payload and returns the response data of type T.
    ///
    /// This asynchronous function constructs a POST request using the `default_post_requestor` method with the given endpoint
    /// and JSON payload. It then sends the request using the request method, expecting a response of type `T` or an error of type `E`.
    /// The error is resolved using the `resolve_error` method and returns an `Option<T>` representing the response data if successful,
    /// or `None` if an error occurred.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint URL to send the POST request to.
    /// * `json` - A string containing the JSON payload to include in the request.
    /// * `map` - A closure that maps the successful response JSON into the desired output type.
    /// * `error_handler` - A closure that handles the error case if an `RequestError` occurs.
    ///
    /// # Returns
    ///
    /// An `Option<O>` representing the response data if successful, or `None` if an error occurred.
    async fn post_request_handler(&self,endpoint : &str,json : String,map : impl FnOnce(T) -> O + Send + Sync,error_handler : impl Fn(RequestError<E>) + Sync  + Send) -> Option<O> {
        let request = self.default_post_requestor(endpoint,json);
        let response = Self::request_map(request,map).await;
        self.resolve_error(response,error_handler)
    }
}


/// Enum representing different types of HTTPS errors.
#[derive(ErrorMacro)]
pub enum RequestError<E> {
    /// Error that occurs when sending a request.
    #[error(
r#"Failed operation relating to request to ({}) with status code of {}. 
Is request: {}, 
Is connect: {}, 
Is body: {}"#,
.0.url().map(|x|x.to_string()).unwrap_or(String::from("Not Found")),
.0.status().map(|x|x.to_string()).unwrap_or(String::from("Not Found")),
.0.is_request(),
.0.is_connect(),
.0.is_body()
)]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse json due to {}",.0)]
    /// Error indicating invalid JSON body during deserialization.
    InvalidJsonBody(#[from] serde_json::Error),

    /// Error payload (json) when request is not successful
    #[error("Request error playload : {0}")]
    ErrorPayload(#[source] E),
}
