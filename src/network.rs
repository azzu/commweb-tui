use reqwest::blocking::{Client, Response};
use reqwest::Method;

pub fn request_url(http_method: Method, url: String) -> Response {
    let client = Client::new();

    let resp = client
        .request(http_method, url)
        .send()
        .expect("Unable to get response.")
        ;

    resp
}