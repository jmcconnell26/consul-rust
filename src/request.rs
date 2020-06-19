use reqwest::blocking::Client as HttpClient;
use reqwest::blocking::RequestBuilder;

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::collections::HashMap;
use std::str;
use std::time::Instant;

use url::Url;

use crate::errors::{Result, ResultExt};
use crate::{Config, QueryOptions, WriteMeta, WriteOptions};

pub mod delete_requests;
pub mod get_requests;
pub mod post_requests;
pub mod put_requests;

pub fn add_config_options(builder: RequestBuilder, config: &Config) -> RequestBuilder {
    match &config.token {
        Some(val) => builder.header("X-Consul-Token", val),
        None => builder,
    }
}

fn construct_write_request_builder<T: Serialize, R: DeserializeOwned, F>(
    path: &str,
    body: Option<&T>,
    config: &Config,
    params: HashMap<String, String>,
    request_builder_from_http_client: F,
) -> Result<RequestBuilder>
where
    F: Fn(&HttpClient, Url) -> RequestBuilder,
{
    let url_str = format!("{}{}", config.address, path);
    let url =
        Url::parse_with_params(&url_str, params.iter()).chain_err(|| "Failed to parse URL")?;

    let builder = request_builder_from_http_client(&config.http_client, url);
    let builder = if let Some(b) = body {
        builder.json(b)
    } else {
        builder
    };
    let builder = add_config_options(builder, &config);

    Ok(builder)
}

fn update_params_with_query_options(
    config: &Config,
    params: &mut HashMap<String, String>,
    options: Option<&QueryOptions>,
) {
    let datacenter: Option<&String> = options
        .and_then(|o| o.datacenter.as_ref())
        .or_else(|| config.datacenter.as_ref());

    if let Some(dc) = datacenter {
        params.insert(String::from("dc"), dc.to_owned());
    }
    if let Some(options) = options {
        if let Some(index) = options.wait_index {
            params.insert(String::from("index"), index.to_string());
        }
        if let Some(wait_time) = options.wait_time {
            params.insert(String::from("wait"), format!("{}s", wait_time.as_secs()));
        }
    }
}

pub fn write_with_body<T: Serialize, R: DeserializeOwned, F>(
    path: &str,
    body: Option<&T>,
    config: &Config,
    mut params: HashMap<String, String>,
    options: Option<&WriteOptions>,
    request_builder_from_http_client: F,
) -> Result<(R, WriteMeta)>
where
    F: Fn(&HttpClient, Url) -> RequestBuilder,
{
    let start = Instant::now();

    let datacenter: Option<&String> = options
        .and_then(|o| o.datacenter.as_ref())
        .or_else(|| config.datacenter.as_ref());

    if let Some(dc) = datacenter {
        params.insert(String::from("dc"), dc.to_owned());
    }

    let builder = construct_write_request_builder::<T, R, F>(
        path,
        body,
        config,
        params,
        request_builder_from_http_client,
    );

    builder?
        .send()
        .chain_err(|| "HTTP request to consul failed")
        .and_then(|x| x.json().chain_err(|| "Failed to parse JSON"))
        .map(|x| {
            (
                x,
                WriteMeta {
                    request_time: Instant::now() - start,
                },
            )
        })
}

#[cfg(test)]
pub mod request_tests {

    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::time::Duration;

    fn setup() -> (RequestBuilder, String) {
        let client = HttpClient::new();
        let builder = client.get("http://127.0.0.1");

        let expected_token: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();

        return (builder, expected_token);
    }

    #[test]
    fn add_config_options_none_test() {
        let (mut builder, _) = setup();

        let mut config = Config::new().unwrap();
        config.token = None;

        builder = add_config_options(builder, &config);

        let request = builder.build().unwrap();
        let headers = request.headers();

        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn add_config_options_some_config_token_test() {
        let (mut builder, expected_token) = setup();

        let mut config = Config::new().unwrap();
        config.token = Some(expected_token.clone());

        builder = add_config_options(builder, &config);

        let request = builder.build().unwrap();
        let headers = request.headers();

        assert_eq!(headers.len(), 1);
        assert!(headers.contains_key("x-consul-token"));
        assert_eq!(headers["x-consul-token"], expected_token);
    }

    #[test]
    fn construct_write_request_builder_with_params_and_no_body_test() {
        let path = "api/";
        let body = None;

        let mut config = Config::new().unwrap();
        config.address = String::from("http://127.0.0.1:8500/");

        let mut params = HashMap::<String, String>::new();
        params.insert(String::from("ParamName"), String::from("ParamValue"));

        let request_builder_from_http_client =
            |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.put(url) };

        let builder = construct_write_request_builder::<String, String, _>(
            path,
            body,
            &config,
            params,
            request_builder_from_http_client,
        )
        .unwrap();

        let request = builder.build().unwrap();

        assert_eq!(
            request.url(),
            &Url::parse("http://127.0.0.1:8500/api/?ParamName=ParamValue").unwrap()
        );
        assert!(request.body().is_none());
    }

    #[test]
    fn construct_write_request_builder_with_no_params_and_body_test() {
        let raw_body = String::from("body");
        let path = "api/";
        let body = Some(&raw_body);

        let mut config = Config::new().unwrap();
        config.address = String::from("http://127.0.0.1:8500/");

        let params = HashMap::<String, String>::new();
        let request_builder_from_http_client =
            |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.put(url) };

        let builder = construct_write_request_builder::<String, String, _>(
            path,
            body,
            &config,
            params,
            request_builder_from_http_client,
        )
        .unwrap();

        let request = builder.build().unwrap();

        assert_eq!(
            request.url(),
            &Url::parse("http://127.0.0.1:8500/api/?").unwrap()
        );

        let body = request.body().unwrap();
        let body_bytes = body.as_bytes().unwrap();
        let body_string = str::from_utf8(body_bytes).unwrap();

        assert_eq!("\"body\"", body_string);
    }

    #[test]
    fn update_params_with_query_options_all_options_test() {
        let config = Config::new().unwrap();
        let mut params = HashMap::<String, String>::new();
        let query_options = QueryOptions {
            datacenter: Some(String::from("test_datacenter")),
            wait_index: Some(123),
            wait_time: Some(Duration::new(5, 0)),
        };

        update_params_with_query_options(&config, &mut params, Some(&query_options));

        assert_eq!(params.len(), 3);
        assert_eq!(params.get("dc").unwrap(), "test_datacenter");
        assert_eq!(params.get("index").unwrap(), "123");
        assert_eq!(params.get("wait").unwrap(), "5s");
    }

    #[test]
    fn update_params_with_query_options_no_options_test() {
        let config = Config::new().unwrap();
        let mut params = HashMap::<String, String>::new();
        let query_options = QueryOptions {
            datacenter: None,
            wait_index: None,
            wait_time: None,
        };

        update_params_with_query_options(&config, &mut params, Some(&query_options));

        assert_eq!(params.len(), 0);
    }
}
