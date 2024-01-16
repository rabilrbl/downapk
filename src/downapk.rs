use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use scraper::{Html, Selector};
use serde_json::{json, Value};

pub struct ApkMirror {
    client: Client,
    host: String,
}

impl ApkMirror {
    pub async fn new() -> Self {
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

        let url = "https://www.apkmirror.com/".to_string();
        let res = client.get(&url).send().await.unwrap().text().await.unwrap();

        let document = Html::parse_document(&res);

        let selector = Selector::parse("button[class='searchButton']").unwrap();

        assert_eq!(1, document.select(&selector).count());

        ApkMirror { client, host: url }
    }

    fn absolute_url(&self, url: &str) -> String {
        if url.starts_with("http") {
            url.to_string()
        } else {
            self.host.to_string() + url
        }
    }

    pub async fn search(&self, search_query: &str) -> Result<Value, Error> {
        let url = format!(
            "https://www.apkmirror.com/?post_type=app_release&searchtype=apk&s={}",
            search_query
        );
        let res = self.client.get(&url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let selector = Selector::parse("a[class='fontBlack']").unwrap();

        let mut results: Value = json!([]);

        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            let link = self.absolute_url(element.value().attr("href").unwrap());

            results.as_array_mut().unwrap().push(json!({
                "title": text,
                "link": link,
            }));
        }

        Ok(results)
    }

    pub async fn download(&self, url: &str) -> Result<Value, Error> {
        let res = self.client.get(url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let table_row_selector = Selector::parse("div[class='table-row headerFont']").unwrap();
        let table_head_selector =
            Selector::parse("div[class='table-cell rowheight addseparator expand pad dowrap']")
                .unwrap();
        let span_apkm_badge_selector = Selector::parse("span[class='apkm-badge']").unwrap();
        let a_accent_color_download_button_selector =
            Selector::parse("a[class='accent_color']").unwrap();
        let mut results: Value = json!([]);

        for table_row_element in document.select(&table_row_selector) {
            for table_head_element in table_row_element.select(&table_head_selector) {
                let badge_text = table_head_element
                    .select(&span_apkm_badge_selector)
                    .next()
                    .map(|element| element.text().collect::<String>())
                    .unwrap_or_default();

                let anchor_elem = table_head_element
                    .select(&a_accent_color_download_button_selector)
                    .next()
                    .unwrap();
               
                let version = anchor_elem.text().collect::<String>().trim().to_string();
                let download_link =
                    self.host.to_string() + anchor_elem.value().attr("href").unwrap();
                
                println!("{:?}", json!({
                    "version": version,
                    "download_link": download_link,
                    "type": badge_text,
                }));

                results.as_array_mut().unwrap().push(json!({
                    "version": version,
                    "download_link": download_link,
                    "type": badge_text,
                }));
            }
        }
        Ok(results)
    }

    // ... other methods here ...
}
