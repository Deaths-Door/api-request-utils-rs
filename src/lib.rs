#![doc = include_str!("../README.md")]

#![forbid(missing_docs)]

use std::collections::HashMap;

pub use reqwest;
pub use serde_json;
pub use serde;
pub use async_trait;

/// A HashMap type used for storing parameters with optional values.
/// The keys are string references, and the values are optional string references.
pub type ParameterHashMap<'a> = HashMap<&'a str,serde_json::Value>;

/// Alias for the `reqwest::Error` type.
pub type ReqwestError = reqwest::Error;

/// Alias for the `serde_json::Error` type.
pub type DeserializationError = serde_json::Error;

/// Alias for a closure type that takes a `RequestError<E>` as an argument and returns `()`.
///
/// The closure should be both `Send` and `Sync`.
pub type ErrorHandler<E> = dyn Fn(RequestError<E>) + Sync;

/// Enum representing different types of request errors.
pub enum RequestError<E> {
    /// Request sending error variant.
    RequestingError(ReqwestError),

    /// Error indicating inability to read the response body.
    UnableToReadBody,

    /// Error indicating invalid JSON body during deserialization.
    InvalidJsonBody(DeserializationError),

    /// Json when request is not successful (default when status code is not 200)
    Json(E),
}

/// A trait for handling HTTP requests.
#[async_trait::async_trait]
pub trait RequestHandler: RequestDefaults {
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
            Err(error) => return Err(RequestError::RequestingError(error)),
            Ok(response) => {
                let status = response.status();
                let body_result = response.text().await;

                match body_result {
                    Err(_) => return Err(RequestError::UnableToReadBody),
                    Ok(body) => {
                        match status.is_success() {
                            true => return Ok(serde_json::from_str(&body).unwrap()),
                            false => return Err(RequestError::Json(serde_json::from_str(&body).unwrap())),
                        }
                    }
                };
            }
        };
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
    fn resolve_error<T,E>(&self,response : Result<T,E>,error_handler: &dyn Fn(E)) -> Option<T> {
        match response {
            Ok(value) => Some(value),
            Err(error) => {
                error_handler(error);
                None
            }
        }
    }
    /// Handles a GET request to the specified endpoint with the provided parameters and returns the response data of type `T`.
    /// 
    /// This asynchronous function constructs (by default) a GET request using the `default_get_requestor` method with the given endpoint and parameters. 
    /// It then sends the request using the request method, expecting a response of type `T` or an error of type `E`.
    /// The error is resolved using the `resolve_error` method and returns an `Option<T>` representing the response data if successful,
    /// or `None` if an error occurred.
    /// 
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint URL to send the GET request to.
    /// * `parameters` - A hashmap containing any parameters to include in the request.
    ///
    async fn get_request_handler<'l,T,E>(&self,endpoint : &str,parameters : ParameterHashMap<'l>,error_handler : &ErrorHandler<E>) -> Option<T> where  T: serde::de::DeserializeOwned, E: serde::de::DeserializeOwned {
        let request = self.default_get_requestor(endpoint,parameters);
        let response = Self::request::<T,E>(request).await;
        self.resolve_error(response,&error_handler)
    }

    /// Handles a POST request to the specified endpoint with the provided JSON payload and returns the response data of type T.
    ///
    /// This asynchronous function constructs a POST request using the `default_post_requestor` method with the given endpoint and json payload. 
    /// It then sends the request using the request method, expecting a response of type `T` or an error of type `E`
    /// The error is resolved using the `resolve_error` method and returns an `Option<T>` representing the response data if successful, 
    /// or `None` if an error occurred.
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint URL to send the POST request to.
    /// * `json` - A string containing the JSON payload to include in the request.
    ///
    async fn post_request_handler<T,E>(&self,endpoint : &str,json : &str,error_handler : &ErrorHandler<E>) -> Option<T> where  T: serde::de::DeserializeOwned, E: serde::de::DeserializeOwned {
        let request = self.default_post_requestor(endpoint,json);
        let response = Self::request::<T,E>(request).await;
        self.resolve_error(response,&error_handler)
    }

} 

/// This trait provides methods for modifying the struct in a specific way:
pub trait RequestModifiers: RequestInfo  {
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
}

/// The RequestDefaults trait provides default methods for configuring and modifying HTTP requests.
pub trait RequestDefaults: RequestModifiers {
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

/// Trait to provide some basic info about API
pub trait RequestInfo {
    /// The base URL for the requests.
    const BASE_URL : &'static str;
}
