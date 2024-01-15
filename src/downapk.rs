use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use serde_json::Value;
// use std::collections::HashMap;

pub struct HttpRequester {
   client: Client,
}

impl HttpRequester {
   pub fn new() -> Self {
       let mut headers = HeaderMap::new();
       headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
       headers.insert("APIKEY", HeaderValue::from_static("6c095e070d272503ebaadda44dd25c3fa0679a39f5059c204d4f0b033e5b695b"));
       headers.insert("Connection", HeaderValue::from_static("Keep-Alive"));
       headers.insert("Host", HeaderValue::from_static("secure.uptodown.com"));
       headers.insert("Identificador", HeaderValue::from_static("Uptodown_Android"));
       headers.insert("Identificador-Version", HeaderValue::from_static("563"));
       headers.insert("User-Agent", HeaderValue::from_static("Dalvik/2.1.0 (Linux; U; Android 13; Pixel 5 Build/TQ3A.230901.001)"));

       let client = Client::builder()
           .default_headers(headers)
           .build().unwrap();

       HttpRequester { client }
   }

   pub async fn get_app_search(&self, search_query: &str, id_platforma: u32) -> Result<Value, Error> {
       let url = format!("https://secure.uptodown.com/eapi/v2/apps/search/{}?page%5Blimit%5D=30&page%5Boffset%5D=0&id_plataforma={}&lang=en", search_query, id_platforma);
       let res = self.client.get(&url).send().await?.json::<Value>().await?;
       Ok(res)
   }

   // ... other methods here ...
}
