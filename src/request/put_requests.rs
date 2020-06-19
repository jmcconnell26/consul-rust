use crate::request::*;

pub fn put<T: Serialize, R: DeserializeOwned>(
    path: &str,
    body: Option<&T>,
    config: &Config,
    params: HashMap<String, String>,
    options: Option<&WriteOptions>,
) -> Result<(R, WriteMeta)> {
    let request_builder_from_http_client =
        |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.put(url) };
    write_with_body(
        path,
        body,
        config,
        params,
        options,
        request_builder_from_http_client,
    )
}
