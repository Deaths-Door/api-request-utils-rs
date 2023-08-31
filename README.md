# api-request-utils

[![Crates.io](https://img.shields.io/crates/v/api-request-utils)](https://crates.io/crates/api-request-utils-rs)
[![Docs.rs](https://docs.rs/api-request-utils/badge.svg)](https://docs.rs/api-request-utils-rs/)

This library aims to provide a straightforward and efficient solution for making API requests. It is designed to be user-friendly, customizable, and extensible, allowing developers to easily integrate and interact with APIs in their Rust applications.

## Features

- Convenient functions for sending HTTP requests and handling responses.
- Error handling utilities for handling different types of request errors.
- JSON serialization and deserialization helpers.
- Parameter encoding and query string generation utilities.
- Request builder and modifier traits for customization and extensibility.

## Installation

Add the following line to your `Cargo.toml` file:

```toml
api-request-utils = "0.2.4" # Note : Latest version at time of writing
```

## Projects using api-request-utils-rs

Here are some projects that are using `api-request-utils-rs`:

- [musixmatch](https://crates.io/crates/musixmatch)
- [ticketmeister](https://crates.io/crates/ticketmeister)

If you're using `api-request-utils-rs` in your project, feel free to contact me to add it to this list!

# Usage

Before you can start making API requests, you need to create an API client that implements the necessary traits. Here's an example of how you can define and implement the API client struct:

```rust
use api_request_utils::*;

struct MyAPIClient {
    // Define your API client fields here such as the client
}

impl RequestInfo for MyAPIClient {
    const BASE_URL: &'static str = "https://api.example.com"; // Replace with the base url
    fn client(&self) -> &reqwest::Client {
        // Return your reqwest::Client instance here
    }
}

// Note : In most cases the default implementations are enough

impl RequestModifiers for MyAPIClient {} // Implement methods for adding headers, modifying requests, etc.

impl RequestDefaults for MyAPIClient {}  // Implement default headers, parameters, and request builders

impl RequestHandler for MyAPIClient {} // Default settings should be enought

```

### Making a GET Request

To make a GET request, you can use the `get_request_handler` method provided by the `RequestHandler` trait. Here's an example:

```rust
#[tokio::main]
async fn main() {
    let api_client = MyAPIClient::new();
    let parameters: HashMap<&str, serde_json::Value> = /* Define your request parameters */;
    let result = api_client.get_request_handler("endpoint", &parameters, |response| response, |error| {
        // Handle error cases
    }).await;

    match result {
        Some(response_data) => {
            // Process the response data
        }
        None => {
            // Handle the error case
        }
    }
}
```

### Making a POST Request

For making a POST request, you can utilize the `post_request_handler` method similarly. Here's an example:

```rust
#[tokio::main]
async fn main() {
    let api_client = MyAPIClient::new();

    let json_payload : String = /* Define your JSON payload */;
    let result = api_client.post_request_handler("endpoint", json_payload, |response| response, |error| {
        // Handle error cases
    }).await;

    match result {
        Some(response_data) => {
            // Process the response data
        }
        None => {
            // Handle the error case
        }
    }
}

```



### Error Handling

The library provides an `RequestError` enum to handle different types of request errors. You can pattern match on this enum to handle specific error scenarios:

```rust
use api_request_utils::RequestError;

match error {
    RequestError::RequestError(reqwest_error) => {
        // Handle request sending errors
    }
    RequestError::InvalidJsonBody(json_error) => {
        // Handle invalid JSON response body errors
    }
    RequestError::ErrorPayload(custom_error) => {
        // Handle custom error payloads from unsuccessful requests
    }
    RequestError::InvalidJsonBody(serde_json_error) => {
        // Handle invalid josn errors
    }
}
```

Please note that the examples provided here are simplified and serve as a starting point. For comprehensive documentation of the crate, please visit the [crate documentation](https://docs.rs/api-request-utils-rs) for a better understanding of the crate's functionalities and APIs.

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvement, please open an issue or submit a pull request.
