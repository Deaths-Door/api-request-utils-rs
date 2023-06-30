//! # api-request-utils-rs
//! [![Crates.io](https://img.shields.io/crates/v/api-request-utils)](https://crates.io/crates/api-request-utils)
//! [![Docs.rs](https://docs.rs/api-request-utils/badge.svg)](https://docs.rs/api-request-utils)
//! 
//! This library aims to provide a straightforward and efficient solution for making api requests It is designed to be user-friendly, customizable, and extensible, allowing developers to easily integrate and interact with APIs in their Rust applications.
//! 
//! ## Features
//! 
//! - Convenient functions for sending HTTP requests and handling responses.
//! - Error handling utilities for handling different types of request errors.
//! - JSON serialization and deserialization helpers.
//! - Parameter encoding and query string generation utilities.
//! - Request builder and modifier traits for customization and extensibility.
//! 
//! ## Installation
//! 
//! Add the following line to your `Cargo.toml` file:
//! 
//! ```toml
//! api-request-utils = "0.1.6"
//! ```
//! 
//! To enable the export feature and include the specified dependencies `(reqwest,serde_json, serde(with derive))`
//! 
//! ```toml
//! api-request-utils = { version = "0.1.6", features = ["export"]}
//! ```
//! 
//! ## Usage
//! 
//! Import the required modules and types in your Rust code:
//! ```rust
//! use api_request_utils::{
//!    ParameterHashMap,
//!     RequestError,
//!     RequestHandler,
//!     RequestInfo
//!     };
//! ```
//! 
//! Then implement the `RequestInfo` trait for your API client struct. Trait to provide some basic info about API : 
//! 
//! ```rust
//! struct MyApiClient;
//! 
//! impl RequestInfo for MyApiClient {
//!     ...
//! }
//! ```
//! 
//! Then implement the `RequestModifiers` trait for your API client struct. This trait provides methods for modifying the struct in a specific way:
//! 
//! ```rust
//! impl RequestModifiers for MyApiClient {
//!     ...
//! }
//! ```
//! 
//! Then implement the `RequestHandler` trait for your API client struct. This trait provides the request method for sending HTTP requests :
//! 
//! ```rust
//! impl RequestHandler for MyApiClient {
//!     ...
//! }
//! ```
//! 
//! Now just combine the methods , data and parameters / json to make the request and handle the error
//! 
//! Please note that the examples provided here are simplified and serve as a starting point. For comprehensive documentation of the crate, please visit the [crate documentation](https://docs.rs/api-request-utils-rs) for a better understanding of the crate's functionalities and APIs.
//! 
//! ## Contributing
//! Contributions are welcome! If you find any issues or have suggestions for improvement, please open an issue or submit a pull request.

use std::collections::HashMap;

#[cfg(feature = "export")]
pub use reqwest;
#[cfg(feature = "export")]
pub use serde_json;
#[cfg(feature = "export")]
pub use serde;

/// A HashMap type used for storing parameters with optional values.
/// The keys are string references, and the values are optional string references.
pub type ParameterHashMap<'a> = HashMap<&'a str, Option<&'a str>>;

/// Enum representing different types of request errors.
///
/// The `Internal` variant represents internal errors with an associated string message.
/// The `Json` variant represents errors related to JSON deserialization with an associated error value.
pub enum RequestError<E> {
    /// Internal error variant with an associated string message.
    Internal(String),
    /// JSON error variant with an associated error value.
    Json(E),
}

/// A trait for handling HTTP requests.
#[async_trait::async_trait]
pub trait RequestHandler {
    /// Sends a request using the given RequestBuilder and handles the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() {
    ///     let url = "https://api.example.com";
    ///     let request = reqwest::Client::new().get(url);
    ///
    ///     match request::<serde_json::Value, serde_json::Value>(request).await {
    ///         Ok(response) => {
    ///             println!("Response: {:?}", response);
    ///         }
    ///         Err(error) => {
    ///             match error {
    ///                 RequestError::Internal(message) => {
    ///                     eprintln!("Internal Error: {}", message);
    ///                 }
    ///                 RequestError::Json(error_data) => {
    ///                     eprintln!("JSON Error: {:?}", error_data);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    async fn request<T,E>(request: reqwest::RequestBuilder) -> Result<T,RequestError<E>> where T : serde::de::DeserializeOwned , E : serde::de::DeserializeOwned {
        let response_result = request.send().await;
        match response_result {
            Err(error) => return Err(RequestError::Internal(error.to_string())),
            Ok(response) => {
                let status = response.status();
                let body_result = response.text().await;

                if body_result.is_err() {
                    return Err(RequestError::Internal("Error in reading response body".to_string()));
                };
                
                let body_string = body_result.unwrap();

                match status.is_success() {
                    true => return Ok(serde_json::from_str(&body_string).unwrap()),
                    false => return Err(RequestError::Json(serde_json::from_str(&body_string).unwrap())),
                }
            }
        };
    }
} 

/// This trait provides methods for modifying the struct in a specific way:
pub trait RequestModifiers : RequestInfo  {
    /// Adds an Authorization header to the given RequestBuilder with the provided token.
    ///
    /// The Authorization header follows the format "Bearer TOKEN", where TOKEN is the
    /// authentication token used for authorization.
    ///
    /// # Arguments
    ///
    /// * request_builder - The RequestBuilder to add the header to.
    /// * token - The authentication token to include in the Authorization header.
    ///
    /// # Returns
    ///
    /// The modified RequestBuilder with the Authorization header added.
    ///
    /// # Example
    ///
    /// ```rust 
    /// use reqwest::RequestBuilder;
    /// let request_builder = reqwest::Client::new().get("https://example.com"); 
    /// let token = "YOUR_AUTH_TOKEN";
    /// let modified_request_builder = authorization_header(&request_builder, token);
    /// ```
    fn authorization_header(request_builder : reqwest::RequestBuilder,token : &str) -> reqwest::RequestBuilder {
        request_builder.header("Authorization",format!(" Bearer {}",token))
    }

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
    /// struct MyStruct;
    /// impl RequestHandler for ... {
    ///     const BASE_URL: &'static str = "https://api.example.com";
    /// }
    /// fn main(){
    ///    let url =  MyStruct::create_endpoint("get");
    ///    assert_eq!(url,"https://api.example.com/get"); // using the default implementation
    /// }
    /// ```
    fn create_endpoint(endpoint : &str) -> String {
        format!("{}/{}",Self::BASE_URL,endpoint)
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
    ///
    /// # Example
    ///
    /// ```rust
    /// fn handle_error(error: &AuthErrorInfo) {
    ///     // Custom error handling logic
    ///     // ...
    /// }
    ///
    /// let response: Result<i32, AuthError> = /* Some API response */;
    /// let result = resolve_error(&response, handle_error);
    ///
    /// match result {
    ///     Some(value) => {
    ///         // Process the value
    ///         // ...
    ///     }
    ///     None => {
    ///         // Error occurred, handle accordingly
    ///         // ...
    ///     }
    /// }
    /// ```
    ///
    /// In the example above, the `resolve_error` function takes a `response` of type `Result<T, E>`,
    /// where `T` represents the success type and `E` represents the error type. It also accepts an
    /// `error_resolver` closure of type `Fn(&E)`, which is responsible for handling the error and
    /// performing custom error handling logic.
    ///
    /// If the response is successful (`Ok` variant), the function returns `Some(value)`, containing
    /// the value. If the response is an error (`Err` variant), the `error_resolver` closure is invoked
    /// with the error as the argument, and `None` is returned.
    fn resolve_error<T,E,F>(response : &Result<T,E>,error_resolver: F) -> Option<&T> where F: Fn(&E) {
        match response {
            Ok(value) => Some(value),
            Err(error) => {
                error_resolver(error);
                None
            }
        }
    }

}

pub trait RequestDefaults : RequestModifiers {
    /// Returns the reqwest::Client instance associated with the API client.
    ///
    /// The client is used to send HTTP requests to the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn main() {
    ///     let api_client = APIClient::new();
    ///     let client = api_client.client();
    ///
    ///     // Use the client to make HTTP requests
    ///     // ...
    /// }
    fn client(&self) -> &reqwest::Client;

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
    fn default_post_requestor(&self,endpoint : &str, json : &str) -> reqwest::RequestBuilder {
        self.default_parameters(self.default_headers(self.client().post(Self::create_endpoint(endpoint)))).body(json.to_owned())
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
    fn default_get_requestor<'a>(&self,endpoint : &str,parameters : ParameterHashMap<'a>) -> reqwest::RequestBuilder {
        self.default_parameters(self.default_headers(self.client().get(Self::create_endpoint(endpoint)))).query(&parameters)
    }
}

// Trait to provide some basic info about API
pub trait RequestInfo {
    /// The base URL for the requests.
    const BASE_URL : &'static str;
}