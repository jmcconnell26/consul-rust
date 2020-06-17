use reqwest::blocking::Client as HttpClient;
use reqwest::blocking::RequestBuilder;

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::collections::HashMap;
use std::str;
use std::time::Instant;

use url::Url;

use crate::errors::{Result, ResultExt};
use crate::{Config, WriteMeta, WriteOptions};

pub mod get_requests;

pub fn add_config_options(builder: RequestBuilder, config: &Config) -> RequestBuilder {
    match &config.token {
        Some(val) => builder.header("X-Consul-Token", val),
        None => builder,
    }
}


pub fn delete<R: DeserializeOwned>(
    path: &str,
    config: &Config,
    params: HashMap<String, String>,
    options: Option<&WriteOptions>,
) -> Result<(R, WriteMeta)> {
    let req = |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.delete(url) };
    write_with_body(path, None as Option<&()>, config, params, options, req)
}

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
pub fn put<T: Serialize, R: DeserializeOwned>(
    path: &str,
    body: Option<&T>,
    config: &Config,
    params: HashMap<String, String>,
    options: Option<&WriteOptions>,
) -> Result<(R, WriteMeta)> {
    let req = |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.put(url) };
    write_with_body(path, body, config, params, options, req)
}

fn write_with_body<T: Serialize, R: DeserializeOwned, F>(
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
