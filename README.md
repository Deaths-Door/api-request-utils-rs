# api-request-utils-rs
[![Crates.io](https://img.shields.io/crates/v/api-request-utils)](https://crates.io/crates/api-request-utils-rs)
[![Docs.rs](https://docs.rs/api-request-utils/badge.svg)](https://docs.rs/api-request-utils-rs)

This library aims to provide a straightforward and efficient solution for making api requests It is designed to be user-friendly, customizable, and extensible, allowing developers to easily integrate and interact with APIs in their Rust applications.

## Features

- Convenient functions for sending HTTP requests and handling responses.
- Error handling utilities for handling different types of request errors.
- JSON serialization and deserialization helpers.
- Parameter encoding and query string generation utilities.
- Request builder and modifier traits for customization and extensibility.

## Installation

Add the following line to your `Cargo.toml` file:

```toml
api-request-utils = "0.2.0"
```

## Usage

Import the required modules and types in your Rust code:
```rust
use api_request_utils::{
    ParameterHashMap,
    RequestError,
    RequestHandler,
    RequestInfo
    };
```

Then implement the `RequestInfo` trait for your API client struct. Trait to provide some basic info about API : 

```rust
struct MyApiClient;

impl RequestInfo for MyApiClient {
    ...
}
```

Then implement the `RequestModifiers` trait for your API client struct. This trait provides methods for modifying the struct in a specific way:

```rust
impl RequestModifiers for MyApiClient {
    ...
}
```

Then implement the `RequestHandler` trait for your API client struct. This trait provides the request method for sending HTTP requests :
```rust
impl RequestHandler for MyApiClient {
    ...
}
```

Now just combine the methods , data and parameters / json to make the request and handle the error

Please note that the examples provided here are simplified and serve as a starting point. For comprehensive documentation of the crate, please visit the [crate documentation](https://docs.rs/api-request-utils-rs) for a better understanding of the crate's functionalities and APIs.

## Contributing
Contributions are welcome! If you find any issues or have suggestions for improvement, please open an issue or submit a pull request.