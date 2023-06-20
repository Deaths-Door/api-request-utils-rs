use std::collections::HashMap;
use reqwest::{Client,StatusCode,RequestBuilder};

pub type ParameterHashMap<'a> = HashMap<&'a str, Option<&'a str>>;

/// A trait for handling HTTP requests.
#[async_trait::async_trait]
pub trait RequestHandler<'a> {
    /// The base URL for the requests.
    const BASE_URL : &'static str;
    /// The API key as string used for authentication.
    const API_KEY : &'static str = "apiKey";

    /// Returns the HTTP client.
    fn client(&self) -> &Client;

    /// Returns the API key.
    fn api_key(&self) -> &'a str;

    /// Handles an error response with the given status code.
    fn on_error(&self,status_code: StatusCode);
    
    /// Concentrates the base URL and endpoint into a complete URL.
    fn concentrate_endpoint(endpoint : &str) -> String {
        format!("{}/{}?",Self::BASE_URL,endpoint)
    }

    /// Builds a request using the provided endpoint and parameters.
    //TODO add get,push variants from https://dtantsur.github.io/rust-openstack/reqwest/struct.Method.html3
    fn build_request(&self,endpoint : &str,parameters : &ParameterHashMap<'a>) -> RequestBuilder {
        self.client().get(Self::concentrate_endpoint(endpoint)).query(&parameters)
    }

    /// Builds the parameter hashmap using the given function.
    fn build_parameters<Function>(&self,function: Function) ->  ParameterHashMap<'a> where Function : FnOnce(&mut ParameterHashMap<'a>) {
        let mut parameters : ParameterHashMap<'a> = HashMap::new();
        parameters.insert(Self::API_KEY,Some(self.api_key()));
        function(&mut parameters);
        parameters
    }

    /// Sends an HTTP request with the given endpoint and parameters, and returns the parsed response.
    async fn request<T>(&self,endpoint: &str,parameters : ParameterHashMap<'a>) -> Result<T, ()> where T : for<'de> serde::Deserialize<'de> {
        let response = self.build_request(endpoint,&parameters)
            .send()
            .await
            .expect("Error in sending Https Request");

        let status = response.status();

        if !status.is_success() {
            self.on_error(status);
            return Err(())
        }

        let body = response
            .text()
            .await
            .expect("Error reading response body");

        let result : T = serde_json::from_str(&body).expect("Error deserializing response body");
        Ok(result)
    }
}