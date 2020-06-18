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
pub mod put_requests;

pub fn add_config_options(builder: RequestBuilder, config: &Config) -> RequestBuilder {
    match &config.token {
        Some(val) => builder.header("X-Consul-Token", val),
        None => builder,
    }
}

//fn  construct_write_request_builder<T: Serialize, R: DeserializeOwned>(
//    path: &str,
//    body: Option<&T>,
//    config: &Config,
//    mut params: HashMap<String, String>,
//    options: Option<&WriteOptions>,
//) -> RequestBuilder {
//}

/*
pub fn post<T: Serialize, R: DeserializeOwned>(path: &str,
                                               body: Option<&T>,
                                               config: &Config,
                                               options: Option<&WriteOptions>)
                                               -> Result<(R, WriteMeta)> {
    let req = |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.post(url) };
    write_with_body(path, body, config, options, req)
}
*/

fn update_params_with_query_options(
    config: &Config,
    params: &mut HashMap<String, String>,
    options: Option<&QueryOptions>) {
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
    req: F,
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

    let url_str = format!("{}{}", config.address, path);
    let url =
        Url::parse_with_params(&url_str, params.iter()).chain_err(|| "Failed to parse URL")?;
    let builder = req(&config.http_client, url);
    let builder = if let Some(b) = body {
        builder.json(b)
    } else {
        builder
    };
    let builder = add_config_options(builder, &config);
    builder
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

    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    use std::time::Duration;
    use super::*;

    fn setup() -> (RequestBuilder, String) {
        let client = HttpClient::new();
        let builder = client.get("http://127.0.0.1");

        let expected_token: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect();

        return (builder, expected_token);
    }

    #[test]
    fn add_config_options_none()
    {
        let (mut builder, _) = setup();

        let mut config = Config::new().unwrap();
        config.token = None;

        builder = add_config_options(builder, &config);

        let request = builder.build().unwrap();
        let headers = request.headers();

        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn add_config_options_some_config_token()
    {
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
    fn update_params_with_query_options_test_all_options() {
        let config = Config::new().unwrap();
        let mut params = HashMap::<String, String>::new();
        let query_options = QueryOptions {
            datacenter: Some(String::from("test_datacenter")),
            wait_index: Some(123),
            wait_time:  Some(Duration::new(5,0)),
        };
        
        update_params_with_query_options(&config, &mut params, Some(&query_options));

        assert_eq!(params.len(), 3);
        assert_eq!(params.get("dc").unwrap(), "test_datacenter");
        assert_eq!(params.get("index").unwrap(), "123");
        assert_eq!(params.get("wait").unwrap(), "5s");
    }

    #[test]
    fn update_params_with_query_options_no_options() {
        let config = Config::new().unwrap();
        let mut params = HashMap::<String, String>::new();
        let query_options = QueryOptions {
            datacenter: None,
            wait_index: None,
            wait_time:  None,
        };

        update_params_with_query_options(&config, &mut params, Some(&query_options));

        assert_eq!(params.len(), 0);
    }
}
