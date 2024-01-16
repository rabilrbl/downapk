use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};

pub struct HttpRequester {
    client: Client,
}

impl HttpRequester {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
        headers.insert(
            reqwest::header::ACCEPT_ENCODING,
            HeaderValue::from_static("text"),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-IN,en-US;q=0.9,en;q=0.8"),
        );
        headers.insert(
            reqwest::header::HOST,
            HeaderValue::from_static("www.apkmirror.com"),
        );
        headers.insert("Proxy-Connection", HeaderValue::from_static("keep-alive"));
        headers.insert(
            reqwest::header::UPGRADE_INSECURE_REQUESTS,
            HeaderValue::from_static("1"),
        );
        headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Linux; Android 13; Pixel 5 Build/TQ3A.230901.001; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/118.0.0.0 Safari/537.36"));
        headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("cf.vojtechh.apkmirror"),
        );

        let client = Client::builder().default_headers(headers).build().unwrap();

        HttpRequester { client }
    }

    pub async fn index(&self) -> Result<String, Error> {
        let url = "http://www.apkmirror.com/".to_string();
        let res = self.client.get(&url).send().await?.text().await?;
        Ok(res)
    }

    // ... other methods here ...
}
