use reqwest::header::HeaderValue;
use reqwest::StatusCode;

use serde::de::DeserializeOwned;

use std::collections::HashMap;
use std::str;
use std::str::FromStr;
use std::time::Instant;

use url::Url;

use crate::errors::{Result, ResultExt};
use crate::request::*;
use crate::{Config, QueryMeta, QueryOptions};

pub fn get<R: DeserializeOwned>(
    path: &str,
    config: &Config,
    mut params: HashMap<String, String>,
    options: Option<&QueryOptions>,
) -> Result<(R, QueryMeta)> {
    update_params_with_query_options(config, &mut params, options);

    let url_str = format!("{}{}", config.address, path);
    let url =
        Url::parse_with_params(&url_str, params.iter()).chain_err(|| "Failed to parse URL")?;
    let start = Instant::now();
    let request_builder = add_config_options(config.http_client.get(url), &config);
    let response = request_builder.send();
    response
        .chain_err(|| "HTTP request to consul failed")
        .and_then(|r| {
            let x: Option<Result<u64>> =
                r.headers()
                    .get("X-Consul-Index")
                    .map(|bytes: &HeaderValue| -> Result<u64> {
                        bytes
                            .to_str()
                            .chain_err(|| "Failed to parse valid UT8 for last index")
                            .and_then(|s: &str| -> Result<u64> {
                                u64::from_str(s)
                                    .chain_err(|| "Failed to parse valid number for last index")
                            })
                    });
            let j = r.json().chain_err(|| "Failed to parse JSON response")?;
            match x {
                Some(r) => Ok((j, Some(r?))),
                None => Ok((j, None)),
            }
        })
        .map(|x: (R, Option<u64>)| {
            (
                x.0,
                QueryMeta {
                    last_index: x.1,
                    request_time: Instant::now() - start,
                },
            )
        })
}

pub fn get_vec<R: DeserializeOwned>(
    path: &str,
    config: &Config,
    mut params: HashMap<String, String>,
    options: Option<&QueryOptions>,
) -> Result<(Vec<R>, QueryMeta)> {
    update_params_with_query_options(config, &mut params, options);

    let url_str = format!("{}{}", config.address, path);
    let url =
        Url::parse_with_params(&url_str, params.iter()).chain_err(|| "Failed to parse URL")?;
    let start = Instant::now();
    let request_builder = add_config_options(config.http_client.get(url), &config);
    let response = request_builder.send();
    response
        .chain_err(|| "HTTP request to consul failed")
        .and_then(|r| {
            let x: Option<Result<u64>> = r
                .headers()
                .get("X-Consul-Index")
                .map(|value: &HeaderValue| value.as_bytes())
                .map(|bytes| {
                    str::from_utf8(bytes)
                        .chain_err(|| "Failed to parse valid UT8 for last index")
                        .and_then(|s| {
                            u64::from_str(s)
                                .chain_err(|| "Failed to parse valid number for last index")
                        })
                });
            let j = if r.status() != StatusCode::NOT_FOUND {
                r.json().chain_err(|| "Failed to parse JSON response")?
            } else {
                Vec::new()
            };
            match x {
                Some(r) => Ok((j, Some(r?))),
                None => Ok((j, None)),
            }
        })
        .map(|x: (Vec<R>, Option<u64>)| {
            (
                x.0,
                QueryMeta {
                    last_index: x.1,
                    request_time: Instant::now() - start,
                },
            )
        })
}
