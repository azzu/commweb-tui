use reqwest::blocking::{Client, Response};
use reqwest::Method;

pub fn request_url(http_method: Method, url: &str) -> Response {
    let client = Client::new();

    let resp = client
        .request(http_method, url)
        .send()
        ;

    resp.unwrap()
}