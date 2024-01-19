use console::Emoji;
use core::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use scraper::{Html, Selector};
use serde_json::{json, json_internal, Value};
use std::cmp::min;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
static DOWNLOAD_EMOJI: Emoji<'_, '_> = Emoji("📥 ", ":-)");
static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");

pub(crate) struct ApkMirror {
    client: Client,
    host: String,
    spinner: ProgressStyle,
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

        let client = Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .unwrap_or_else(|e| {
                panic!(
                    "Something went wrong while creating reqwest client. Err: {}",
                    e
                )
            });

        let spinner_style =
            ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
                .unwrap_or_else(|e| {
                    panic!(
                        "Something went wrong while creating spinner style. Err: {}",
                        e
                    )
                })
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let pb = ProgressBar::new(40);
        pb.set_style(spinner_style.clone());
        pb.set_prefix(format!(" {} Intialise", SPARKLE));

        pb.set_message("Heading to apkmirror.com for valid cookies");
        pb.enable_steady_tick(Duration::from_millis(100));
        let url = "https://www.apkmirror.com".to_string();
        let res = client
            .get(&(url.clone() + "/"))
            .send()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Something went wrong while making request for cookies. Err: {}",
                    e
                )
            })
            .text()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Something went wrong while unwrapping response text. Err: {}",
                    e
                )
            });

        pb.set_message("Got some cookies, parsing html");
        let document = Html::parse_document(&res);

        pb.set_message("Parsing html to check if page is valid");
        let selector = Selector::parse("button[class='searchButton']").unwrap();

        assert_eq!(1, document.select(&selector).count());

        pb.finish_with_message("Finished getting valid cookies");

        ApkMirror {
            client,
            host: url,
            spinner: spinner_style,
        }
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
        let pb = ProgressBar::new(40);
        pb.set_style(self.spinner.clone());
        pb.set_prefix(format!(" {} Search", LOOKING_GLASS));
        match version {
            Some(version) => {
                pb.set_message(format!("Searching in {} for version {}", url, version))
            }
            None => pb.set_message(format!("Searching in {}", url)),
        }
        pb.enable_steady_tick(Duration::from_millis(100));

        pb.set_message(format!("Making request to {}", url));
        let res = self.client.get(url).send().await?.text().await?;

        pb.set_message("Parsing html");
        let document = Html::parse_document(&res);

        let list_widget_selector = Selector::parse("div.listWidget").unwrap();
        let div_without_class_selector = Selector::parse("div:not([class])").unwrap();
        let link_selector = Selector::parse("a[class='fontBlack']").unwrap();
        let info_selector = Selector::parse("div.infoSlide.t-height").unwrap();
        let paragraph_selector = Selector::parse("p").unwrap();
        let info_name_selector = Selector::parse("span.infoSlide-name").unwrap();
        let info_value_selector = Selector::parse("span.infoSlide-value").unwrap();

        let mut results: Value = json!([]);

        pb.set_message("Processing each APK result");
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
                    Some(link) => self.absolute_url(
                        link.value()
                            .attr("href")
                            .expect("Could not get attribute href"),
                    ),
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
                                    .expect("Could not strip suffix")
                                    .to_string(),
                                None => continue,
                            };

                            let value = match value {
                                Some(value) => {
                                    value.text().collect::<String>().trim().to_string()
                                }
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

                results
                    .as_array_mut()
                    .expect("Could not get mutable results array")
                    .push(temp_result);
            }
        }
        pb.finish_with_message("Finished search");

        Ok(results)
    }

    pub async fn search(&self, search_query: &str) -> Result<Value, Error> {
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
        arch_: Option<&str>,
        dpi: Option<&str>,
    ) -> Result<Value, Error> {
        let pb = ProgressBar::new(40);
        pb.set_style(self.spinner.clone());
        pb.set_prefix(format!(" {} Get file download links", TRUCK));
        pb.set_message(format!("Trying to get all download links from {}", url));
        pb.enable_steady_tick(Duration::from_millis(100));
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

        pb.set_message("Processing each link");
        for table_row_element in document.select(&table_row_selector) {
            pb.set_message("Processing a link");
            for table_head_element in table_row_element.select(&table_head_selector) {
                let badge_text = table_head_element
                    .select(&span_apkm_badge_selector)
                    .next()
                    .map(|element| element.text().collect::<String>())
                    .unwrap_or_default();

                let anchor_elem = match table_head_element
                    .select(&a_accent_color_download_button_selector)
                    .next()
                {
                    Some(anchor_elem) => anchor_elem,
                    None => continue,
                };

                let version = anchor_elem.text().collect::<String>().trim().to_string();

                let download_link = self.absolute_url(
                    anchor_elem
                        .value()
                        .attr("href")
                        .expect("Could not get attribute href"),
                );

                if badge_text != "" && version != "" && download_link != "" {
                    if let Some(type_) = type_ {
                        if type_ != badge_text {
                            pb.set_message(format!("Skipping type {}", badge_text));
                            continue;
                        }
                    }
                    let arch: String = table_row_element
                        .select(metadata_selector)
                        .nth(1)
                        .expect("Could not get arch string")
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string();
                    if let Some(arch_) = arch_ {
                        if arch_ != arch {
                            pb.set_message(format!("Skipping arch {}", arch));
                            continue;
                        }
                    }
                    let screen_dpi = table_row_element
                        .select(metadata_selector)
                        .nth(3)
                        .expect("Could not get screen dpi")
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string();
                    if let Some(dpi) = dpi {
                        if dpi != screen_dpi {
                            pb.set_message(format!("Skipping dpi {}", screen_dpi));
                            continue;
                        }
                    }
                    let min_version = table_row_element
                        .select(metadata_selector)
                        .nth(2)
                        .expect("Could not get min version")
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string();
                    pb.set_message(format!("Found version: {} with type: {} and arch: {} and min_version: {} and screen_dpi: {}", version, badge_text, arch, min_version, screen_dpi));
                    results.as_array_mut().unwrap_or_else(|| panic!("Could not get mutable results array"))
                    .push(json_internal!({
                        "version":version,
                        "download_link":match self.download_link(&download_link, &pb).await {
                            Ok(download_link) => download_link, Err(e) => panic!("Something went wrong while getting download link. Err: {}",e),
                        },
                        "type":badge_text,
                        "arch":arch,
                        "min_version":min_version,
                        "screen_dpi":screen_dpi,
                    }));
                }
            }
        }
        pb.finish_with_message("Finished getting all download links");
        Ok(results)
    }

    pub async fn _download_by_arch(
        &self,
        url: &str,
        arch: Option<&str>,
    ) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, None, arch, None).await?)
    }

    pub async fn _download_by_type(
        &self,
        url: &str,
        type_: Option<&str>,
    ) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, type_, None, None).await?)
    }

    pub async fn _download_by_dpi(&self, url: &str, dpi: Option<&str>) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, None, None, dpi).await?)
    }

    pub async fn _download(&self, url: &str) -> Result<Value, Error> {
        Ok(self.download_by_specifics(url, None, None, None).await?)
    }

    async fn download_link(&self, url: &str, pb: &ProgressBar) -> Result<String, Error> {
        pb.set_message(format!("Trying to get download page link from {}", url));
        let res = self.client.get(url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let selector = Selector::parse("a.accent_bg.btn.btn-flat.downloadButton").unwrap();
        let final_download_link_selector =
            Selector::parse("a[rel='nofollow'][data-google-vignette='false']").unwrap();

        let download_link = document.select(&selector).next();

        let final_download_link = match download_link {
            Some(download_link) => {
                pb.set_message(format!(
                    "Found download link page, trying to get final download link"
                ));
                let download_link =
                    self.absolute_url(download_link.value().attr("href").unwrap());

                let res = self.client.get(download_link).send().await?.text().await?;

                let document = Html::parse_document(&res);

                let final_download_link = document.select(&final_download_link_selector).next();

                match final_download_link {
                    Some(final_download_link) => {
                        let final_download_link = self.absolute_url(
                            final_download_link
                                .value()
                                .attr("href")
                                .unwrap_or_else(|| panic!("Could not get final download link")),
                        );
                        pb.set_message(format!(
                            "Found final download link: {}",
                            final_download_link
                        ));
                        final_download_link.to_string()
                    }
                    None => panic!("No download link found"),
                }
            }
            None => panic!("No download link found"),
        };
        pb.set_message("Finished getting download link");
        Ok(final_download_link)
    }

    // ... other methods here ...
}

pub async fn download_file(
    downlinks: &Vec<Value>,
    package_name: &str,
    output_dir: &str,
) -> Result<(), Error> {
    // if output_dir is not present, create it
    match tokio::fs::create_dir(output_dir).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                panic!(
                    "Something went wrong while creating output directory. Err: {}",
                    e
                );
            }
        }
    };
    for downlink in downlinks {
        let download_link = downlink
            .as_object()
            .unwrap_or_else(|| panic!("Could not get download link"))
            .get("download_link")
            .unwrap()
            .as_str()
            .unwrap_or_else(|| panic!("Could not convert download link to str"));
        let url = download_link;
        let version = downlink
            .as_object()
            .expect("Could not get version")
            .get("version")
            .expect("Could not get key version")
            .as_str()
            .expect("Could not convert version to str");
        let arch = downlink
            .as_object()
            .expect("Could not get arch")
            .get("arch")
            .unwrap()
            .as_str()
            .expect("Could not convert arch to str");
        let dpi = downlink
            .as_object()
            .expect("Could not get screen_dpi")
            .get("screen_dpi")
            .expect("Could not get key screen_dpi")
            .as_str()
            .expect("Could not convert screen_dpi to str");
        let extension = match downlink
            .as_object()
            .expect("Couldn't convert downlink to object")
            .get("type")
            .expect("Couldn't get type")
            .as_str()
            .expect("Couldn't convert type to str")
        {
            "APK" => "apk",
            "BUNDLE" => "apkm",
            ext => panic!("Got an unknown apk type: {}", ext),
        };

        let mut res = reqwest::get(url).await?;
        let total_size = res
            .content_length()
            .ok_or("Failed to get content length")
            .unwrap();

        let pb = ProgressBar::new(total_size);
        pb.set_prefix(format!(" {} Downloading", DOWNLOAD_EMOJI));
        pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());

        let output_file = format!(
            "{}_{}_{}_{}.{}",
            package_name, version, arch, dpi, extension
        );
        let output_path = format!("{}/{}", output_dir, output_file);
        pb.set_message(format!("File {}", output_file));
        let mut file = File::create(output_path)
            .await
            .expect("Failed to create file");

        let mut downloaded: u64 = 0;

        while let Some(chunk) = res.chunk().await.expect("Error while downloading file") {
            file.write_all(&chunk)
                .await
                .expect("Error while writing to file");

            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(format!("Finished downloading file {}", output_file));
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let downloader = ApkMirror::new().await;
        let search_query = "com.google.android.youtube";
        let version = "19.02.34";
        let result = downloader.search_by_version(search_query, version).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_array());
        let object = value[0].as_object().unwrap();
        assert!(object.contains_key("title"));
        assert!(object.contains_key("link"));
        assert!(object.contains_key("Version"));
        assert_eq!(object["Version"], Value::String(version.to_string()));
    }

    #[tokio::test]
    async fn test_extract_root_links() {
        let downloader = ApkMirror::new().await;
        let url = "https://www.apkmirror.com/?post_type=app_release&searchtype=apk&s=com.google.android.youtube";
        let result = downloader.extract_root_links(url, None).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_array());
        for object in value.as_array().unwrap() {
            assert!(object.is_object());
            assert!(object.as_object().unwrap().contains_key("title"));
            assert!(object.as_object().unwrap().contains_key("link"));
        }
    }

    #[tokio::test]
    async fn test_download() {
        let downloader = ApkMirror::new().await;
        let url = "https://www.apkmirror.com/apk/instagram/instagram-lite/instagram-lite-390-0-0-9-116-release/";
        let arch = "armeabi-v7a";
        let type_ = "APK";
        let dpi = "nodpi";
        let result = downloader
            .download_by_specifics(url, Some(type_), Some(arch), Some(dpi))
            .await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_array());
        let array = value.as_array().unwrap();
        assert!(!array.is_empty());
        for item in array {
            assert!(item.is_object());
            let object = item.as_object().unwrap();
            assert!(object.contains_key("version"));
            assert!(object.contains_key("download_link"));
            assert!(object.contains_key("type"));
            assert!(object.contains_key("arch"));
            assert!(object.contains_key("min_version"));
            assert!(object.contains_key("screen_dpi"));
            assert_eq!(object["type"], Value::String(type_.to_string()));
            assert_eq!(object["arch"], Value::String(arch.to_string()));
            assert_eq!(object["screen_dpi"], Value::String(dpi.to_string()));
        }

        match download_file(array, "com.instagram.lite", "downloads").await {
            Ok(val) => {
                assert_eq!(val, ());
                // check if file exists in output directory
                let partial_filename = "com.instagram.lite_";
                // open output directory
                let mut dir = tokio::fs::read_dir("downloads").await.unwrap();
                // iterate over files in output directory
                while let Some(entry) = dir.next_entry().await.unwrap() {
                    // get file name
                    let filename = entry.file_name();
                    // convert filename to string
                    let filename = filename.into_string().unwrap();
                    // check if filename contains partial_filename
                    if filename.contains(partial_filename) {
                        assert!(filename.ends_with(".apk"));
                        // delete file after test
                        tokio::fs::remove_file(format!("downloads/{}", filename))
                            .await
                            .unwrap();
                        break;
                    }
                }
            }
            Err(e) => panic!("Error while downloading file. Err: {}", e),
        }
    }
}
