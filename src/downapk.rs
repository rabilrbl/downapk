use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use scraper::{Html, Selector};
use serde_json::{json, json_internal, Value};

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

        println!("Heading to apkmirror.com for valid cookies");
        let url = "https://www.apkmirror.com".to_string();
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

    pub async fn extract_root_links(
        &self,
        url: &str,
        version: Option<&str>,
    ) -> Result<Value, Error> {
        println!("Trying to get all apk pages from {}", url);

        let res = self.client.get(url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let list_widget_selector = Selector::parse("div.listWidget").unwrap();
        let div_without_class_selector = Selector::parse("div:not([class])").unwrap();
        let link_selector = Selector::parse("a[class='fontBlack']").unwrap();
        let info_selector = Selector::parse("div.infoSlide.t-height").unwrap();
        let paragraph_selector = Selector::parse("p").unwrap();
        let info_name_selector = Selector::parse("span.infoSlide-name").unwrap();
        let info_value_selector = Selector::parse("span.infoSlide-value").unwrap();

        let mut results: Value = json!([]);

        for element in document.select(&list_widget_selector).take(1) {
            for element in element.select(&div_without_class_selector) {
                let mut temp_result = json!({});
                let link = element.select(&link_selector).next();
                let info = element.select(&info_selector).next();

                let text = match link {
                    Some(link) => link.text().collect::<String>(),
                    None => continue,
                };

                let link = match link {
                    Some(link) => self.absolute_url(link.value().attr("href").unwrap()),
                    None => continue,
                };

                match info {
                    Some(info) => {
                        for element in info.select(&paragraph_selector) {
                            let name = element.select(&info_name_selector).next();
                            let value = element.select(&info_value_selector).next();

                            let name = match name {
                                Some(name) => name
                                    .text()
                                    .collect::<String>()
                                    .trim()
                                    .strip_suffix(":")
                                    .unwrap()
                                    .to_string(),
                                None => continue,
                            };

                            let value = match value {
                                Some(value) => value.text().collect::<String>().trim().to_string(),
                                None => continue,
                            };

                            temp_result[name] = Value::String(value);
                        }
                    }
                    None => continue,
                };

                if let Some(version) = version {
                    if temp_result["Version"] != Value::String(version.to_string()) {
                        continue;
                    }
                }

                temp_result["title"] = Value::String(text);
                temp_result["link"] = Value::String(link);

                results.as_array_mut().unwrap().push(temp_result);
            }
        }
        println!("Finished collecting all apk pages");

        Ok(results)
    }

    pub async fn search(&self, search_query: &str) -> Result<Value, Error> {
        println!("Searching for {}", search_query);

        let url = self.absolute_url(&format!(
            "/?post_type=app_release&searchtype=apk&s={}",
            search_query
        ));

        Ok(self.extract_root_links(&url, None).await?)
    }

    pub async fn search_by_version(
        &self,
        search_query: &str,
        version: &str,
    ) -> Result<Value, Error> {
        println!("Searching for {} with version {}", search_query, version);

        let url = self.absolute_url(&format!(
            "/?post_type=app_release&searchtype=apk&s={}",
            search_query
        ));

        Ok(self.extract_root_links(&url, Some(version)).await?)
    }

    pub async fn download_by_specifics(
        &self,
        url: &str,
        type_: Option<&str>,
        arch: Option<&str>,
        dpi: Option<&str>,
    ) -> Result<Value, Error> {
        println!("Trying to get all downloadable links from {}", url);
        let res = self.client.get(url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let table_row_selector = Selector::parse("div[class='table-row headerFont']").unwrap();
        let table_head_selector =
            Selector::parse("div[class='table-cell rowheight addseparator expand pad dowrap']")
                .unwrap();
        let span_apkm_badge_selector = Selector::parse("span.apkm-badge").unwrap();
        let a_accent_color_download_button_selector =
            Selector::parse("a[class='accent_color']").unwrap();
        let metadata_selector = &Selector::parse("div").unwrap();
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
                    .next();

                let version = match anchor_elem {
                    Some(anchor_elem) => anchor_elem.text().collect::<String>().trim().to_string(),
                    None => continue,
                };

                let download_link = match anchor_elem {
                    Some(anchor_elem) => {
                        self.absolute_url(anchor_elem.value().attr("href").unwrap())
                    }
                    None => continue,
                };

                if badge_text != "" && version != "" && download_link != "" {
                    if let Some(type_) = type_ {
                        if type_ != badge_text {
                            println!("Skipping type {}", badge_text);
                            continue;
                        }
                    }
                    let archstr = table_row_element
                        .select(metadata_selector)
                        .nth(1)
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string();
                    if let Some(arch) = arch {
                        if arch != archstr {
                            println!("Skipping arch {}", archstr);
                            continue;
                        }
                    }
                    let screen_dpi = table_row_element
                        .select(metadata_selector)
                        .nth(3)
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string();
                    if let Some(dpi) = dpi {
                        if dpi != screen_dpi {
                            println!("Skipping dpi {}", screen_dpi);
                            continue;
                        }
                    }
                    let min_version = table_row_element
                        .select(metadata_selector)
                        .nth(2)
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string();
                    println!("Found version: {} with type: {} and arch: {} and min_version: {} and screen_dpi: {}", version, badge_text, archstr, min_version, screen_dpi);
                    results.as_array_mut().unwrap().push(json_internal!({
                        "version":version,
                        "download_link":match self.download_link(&download_link).await {
                            Ok(download_link) => download_link, Err(e) => panic!("Something went wrong while getting download link. Err: {}",e),
                        },
                        "type":badge_text,
                        "arch":archstr,
                        "min_version":min_version,
                        "screen_dpi":screen_dpi,
                    }));
                }
            }
        }
        Ok(results)
    }

    pub async fn _download_by_arch(&self, url: &str, arch: Option<&str>) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, None, arch, None).await?)
    }

    pub async fn _download_by_type(&self, url: &str, type_: Option<&str>) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, type_, None, None).await?)
    }

    pub async fn _download_by_dpi(&self, url: &str, dpi: Option<&str>) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, None, None, dpi).await?)
    }

    pub async fn _download(&self, url: &str) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, None, None, None).await?)
    }

    async fn download_link(&self, url: &str) -> Result<String, Error> {
        println!("Trying to get download page link from {}", url);
        let res = self.client.get(url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let selector = Selector::parse("a.accent_bg.btn.btn-flat.downloadButton").unwrap();
        let final_download_link_selector =
            Selector::parse("a[rel='nofollow'][data-google-vignette='false']").unwrap();

        let download_link = document.select(&selector).next();

        let final_download_link = match download_link {
            Some(download_link) => {
                println!("Found download link page, trying to get final download link");
                let download_link = self.absolute_url(download_link.value().attr("href").unwrap());

                let res = self.client.get(download_link).send().await?.text().await?;

                let document = Html::parse_document(&res);

                let final_download_link = document.select(&final_download_link_selector).next();

                match final_download_link {
                    Some(final_download_link) => {
                        let final_download_link =
                            self.absolute_url(final_download_link.value().attr("href").unwrap());
                        println!("Found final download link: {}", final_download_link);
                        final_download_link.to_string()
                    }
                    None => panic!("No download link found"),
                }
            }
            None => panic!("No download link found"),
        };

        Ok(final_download_link)
    }

    // ... other methods here ...
}
