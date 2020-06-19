use crate::request::*;

pub fn _post<T: Serialize, R: DeserializeOwned>(
    path: &str,
    body: Option<&T>,
    config: &Config,
    params: HashMap<String, String>,
    options: Option<&WriteOptions>,
) -> Result<(R, WriteMeta)> {
    let request_builder_from_http_client =
        |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.post(url) };
    write_with_body(
        path,
        body,
        config,
        params,
        options,
        request_builder_from_http_client,
    )
}
