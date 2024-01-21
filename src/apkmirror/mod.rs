use crate::utils::selector;
use console::Emoji;
use core::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use scraper::Html;
use std::cmp::min;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
static DOWNLOAD_EMOJI: Emoji<'_, '_> = Emoji("📥 ", ":-)");
static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");

/// Represents a structure for downloading APK files from ApkMirror.
pub struct DownloadApkMirror {
    /// The version of the APK file.
    pub version: String,
    /// The download link of the APK file.
    pub download_link: String,
    /// The type of the APK file. It can be either APK or BUNDLE.
    pub type_: String,
    /// The architecture of the APK file. It can be either arm64-v8a, armeabi-v7a, x86, x86_64, universal.
    pub arch: String,
    /// The minimum version of Android required to run the APK file.
    pub min_version: String,
    /// The screen dpi of the APK file. It can be either nodpi, 120-640dpi, ...
    pub screen_dpi: String,
}

/// Represents the extracted links from a source.
pub struct ExtractedLink {
    /// The version of the extracted link.
    pub version: String,
    /// The number of downloads for the extracted link.
    pub downloads: String,
    /// The file size of the extracted link.
    pub file_size: String,
    /// The date and time when the link was uploaded.
    pub uploaded: String,
    /// The actual link extracted.
    pub link: String,
    /// The title of the extracted link.
    pub title: String,
}

/// Implements the `Default` trait for the `ExtractedLinks` struct.
/// 
/// This allows creating a default instance of `ExtractedLinks` using the `default()` method.
/// The default instance has all fields initialized with empty strings.
impl Default for ExtractedLink {
    fn default() -> Self {
        ExtractedLink {
            version: String::new(),
            downloads: String::new(),
            file_size: String::new(),
            uploaded: String::new(),
            link: String::new(),
            title: String::new(),
        }
    }
}

/// Represents an ApkMirror instance. This struct contains:
/// - `client`: The reqwest client used to make HTTP requests.
/// - `host`: The host URL of the ApkMirror website.
/// - `spinner`: The progress spinner style for loading animations.
///
/// This is exported as part of the public API.
pub struct ApkMirror {
    /// The reqwest client.
    client: Client,
    /// The host of the ApkMirror instance.
    host: String,
    /// The spinner style for loading animations.
    spinner: ProgressStyle,
}

impl ApkMirror {
    /// Initializes a new instance of `ApkMirror`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use downapk::apkmirror::ApkMirror;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let apk_mirror = ApkMirror::new().await;
    /// }
    /// ```
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

        let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
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
        let selector = selector("button[class='searchButton']");

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

    /// Extracts the root links from the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to extract the root links from.
    /// * `version` - Optional version to filter the results by.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `ExtractedLinks` or an `Error` if the extraction fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use downapk::apkmirror::ApkMirror;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let apk_mirror = ApkMirror::new().await;
    ///     let links = apk_mirror.extract_root_links("https://www.apkmirror.com/apk/instagram/instagram-lite/", Some("390.0.0.9.116")).await;
    /// }
    /// ```
    pub async fn extract_root_links(
        &self,
        url: &str,
        version: Option<&str>,
    ) -> Result<Vec<ExtractedLink>, Error> {
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

        let list_widget_selector = selector("div.listWidget");
        let div_without_class_selector = selector("div:not([class])");
        let link_selector = selector("a[class='fontBlack']");
        let info_selector = selector("div.infoSlide.t-height");
        let paragraph_selector = selector("p");
        let info_name_selector = selector("span.infoSlide-name");
        let info_value_selector = selector("span.infoSlide-value");

        let mut results: Vec<ExtractedLink> = vec![];

        pb.set_message("Processing each APK result");
        for element in document.select(&list_widget_selector).take(1) {
            for element in element.select(&div_without_class_selector) {
                let mut extracted_link = ExtractedLink::default();
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
                                Some(name) => {
                                    let name = name
                                        .text()
                                        .collect::<String>()
                                        .trim()
                                        .strip_suffix(':')
                                        .expect("Could not strip suffix")
                                        .to_owned();
                                    name
                                }
                                None => continue,
                            };

                            let value = match value {
                                Some(value) => value.text().collect::<String>().trim().to_string(),
                                None => continue,
                            };

                            match name.as_str() {
                                "Version" => extracted_link.version = value,
                                "Downloads" => extracted_link.downloads = value,
                                "File Size" => extracted_link.file_size = value,
                                "Uploaded" => extracted_link.uploaded = value,
                                _ => continue,
                            }
                        }
                    }
                    None => continue,
                };

                if let Some(version) = version {
                    if extracted_link.version != version {
                        continue;
                    }
                }

                extracted_link.title = text;
                extracted_link.link = link;

                results.push(extracted_link);
            }
        }
        pb.finish_with_message("Finished search");

        Ok(results)
    }

    /// Searches for APKs on ApkMirror based on the specified search query.
    ///
    /// # Arguments
    ///
    /// * `search_query` - The search query to use.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `ExtractedLinks` or an `Error` if the search fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use downapk::apkmirror::ApkMirror;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let apk_mirror = ApkMirror::new().await;
    ///     let results = apk_mirror.search("com.instagram.lite").await;
    /// }
    /// ```
    pub async fn search(&self, search_query: &str) -> Result<Vec<ExtractedLink>, Error> {
        let url = self.absolute_url(&format!(
            "/?post_type=app_release&searchtype=apk&s={}",
            search_query
        ));

        self.extract_root_links(&url, None).await
    }

    /// Searches for APKs on ApkMirror based on the specified search query and version.
    ///
    /// # Arguments
    ///
    /// * `search_query` - The search query to use.
    /// * `version` - The version to filter the results by.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `ExtractedLinks` or an `Error` if the search fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use downapk::apkmirror::ApkMirror;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let apk_mirror = ApkMirror::new().await;
    ///     let results = apk_mirror.search_by_version("com.instagram.lite", "390.0.0.9.116").await;
    /// }
    /// ```
    pub async fn search_by_version(
        &self,
        search_query: &str,
        version: &str,
    ) -> Result<Vec<ExtractedLink>, Error> {
        let url = self.absolute_url(&format!(
            "/?post_type=app_release&searchtype=apk&s={}",
            search_query
        ));

        self.extract_root_links(&url, Some(version)).await
    }

    /// Downloads APKs from ApkMirror based on the specified URL and optional parameters.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the APK to download.
    /// * `type_` - Optional type of the APK (e.g., arm64-v8a).
    /// * `arch_` - Optional architecture of the APK (e.g., arm64).
    /// * `dpi` - Optional DPI (dots per inch) of the APK.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `DownloadApkMirror` or an `Error` if the download fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use downapk::apkmirror::ApkMirror;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let apk_mirror = ApkMirror::new().await;
    ///     let downloads = apk_mirror.download_by_specifics("https://www.apkmirror.com/apk/instagram/instagram-lite/instagram-lite-390-0-0-9-116-release/", Some("APK"), Some("arm64-v8a"), Some("nodpi")).await;
    /// }
    /// ```
    pub async fn download_by_specifics(
        &self,
        url: &str,
        type_: Option<&str>,
        arch_: Option<&str>,
        dpi: Option<&str>,
    ) -> Result<Vec<DownloadApkMirror>, Error> {
        let pb = ProgressBar::new(40);
        pb.set_style(self.spinner.clone());
        pb.set_prefix(format!(" {} Get file download links", TRUCK));
        pb.set_message(format!("Trying to get all download links from {}", url));
        pb.enable_steady_tick(Duration::from_millis(100));
        let res = self.client.get(url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let table_row_selector = selector("div[class='table-row headerFont']");
        let table_head_selector =
            selector("div[class='table-cell rowheight addseparator expand pad dowrap']");
        let span_apkm_badge_selector = selector("span.apkm-badge");
        let a_accent_color_download_button_selector = selector("a[class='accent_color']");
        let metadata_selector = &selector("div");
        let mut results: Vec<DownloadApkMirror> = vec![];

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

                if !badge_text.is_empty() && !version.is_empty() && !download_link.is_empty() {
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
                    results.push(DownloadApkMirror {
                        version,
                        download_link: match self.download_link(&download_link, &pb).await {
                            Ok(download_link) => download_link,
                            Err(e) => panic!(
                                "Something went wrong while getting download link. Err: {}",
                                e
                            ),
                        },
                        type_: badge_text,
                        arch,
                        min_version,
                        screen_dpi,
                    });
                }
            }
        }
        pb.finish_with_message("Finished getting all download links");
        Ok(results)
    }

    /// Gets the download link of the specified URL with specific arch.
    /// This method is a shorthand for `download_by_specifics(url, None, arch, None)`.
    pub async fn _download_by_arch(
        &self,
        url: &str,
        arch: Option<&str>,
    ) -> Result<Vec<DownloadApkMirror>, Error> {
        self.download_by_specifics(url, None, arch, None).await
    }

    /// Gets the download link of the specified URL with specific type.
    /// This method is a shorthand for `download_by_specifics(url, type_, None, None)`.
    pub async fn _download_by_type(
        &self,
        url: &str,
        type_: Option<&str>,
    ) -> Result<Vec<DownloadApkMirror>, Error> {
        self.download_by_specifics(url, type_, None, None).await
    }

    /// Gets the download link of the specified URL with specific dpi.
    /// This method is a shorthand for `download_by_specifics(url, None, None, dpi)`.
    pub async fn _download_by_dpi(
        &self,
        url: &str,
        dpi: Option<&str>,
    ) -> Result<Vec<DownloadApkMirror>, Error> {
        self.download_by_specifics(url, None, None, dpi).await
    }

    /// Gets the download link of the specified URL without any specific parameters.
    /// This method is a shorthand for `download_by_specifics(url, None, None, None)`.
    pub async fn _download(&self, url: &str) -> Result<Vec<DownloadApkMirror>, Error> {
        self.download_by_specifics(url, None, None, None).await
    }

    /// Gets the final direct file download link from the specified URL.
    /// This method is used internally by `download_by_specifics`.
    /// It is recommended to use `download_by_specifics` instead of this method.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to get the final download link from.
    /// * `pb` - The progress bar to use.
    ///
    /// # Returns
    ///
    /// A `Result` containing the final download link or an `Error` if the download link could not be found.
    async fn download_link(&self, url: &str, pb: &ProgressBar) -> Result<String, Error> {
        pb.set_message(format!("Trying to get download page link from {}", url));
        let res = self.client.get(url).send().await?.text().await?;

        let document = Html::parse_document(&res);

        let download_button_selector = selector("a.accent_bg.btn.btn-flat.downloadButton");
        let final_download_link_selector =
            selector("a[rel='nofollow'][data-google-vignette='false']");

        let download_link = document.select(&download_button_selector).next();

        let final_download_link = match download_link {
            Some(download_link) => {
                pb.set_message("Found download link page, trying to get final download link");
                let download_link = self.absolute_url(download_link.value().attr("href").unwrap());

                let res = self.client.get(download_link).send().await?.text().await?;

                let document = Html::parse_document(&res);

                let final_download_link = document.select(&final_download_link_selector).next();

                match final_download_link {
                    Some(final_download_link) => {
                        let final_download_link = self.absolute_url(
                            final_download_link
                                .value()
                                .attr("href")
                                .expect("Could not get final download link"),
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

/// Downloads multiple APK files from APKMirror based on the provided vector of DownloadApkMirror structs.
/// Iterates over the vector and calls single_file_download for each item.
///
/// # Arguments
///
/// * `downlinks` - The vector of DownloadApkMirror structs to download.
/// * `package_name` - The package name of the APK file.
/// * `output_dir` - The output directory to download the APK files to.
///
/// # Returns
///
/// A `Result` containing `()` or an `Error` if the download fails.
pub async fn multiple_file_download(
    downlinks: &Vec<DownloadApkMirror>,
    package_name: &str,
    output_dir: &str,
) -> Result<(), Error> {
    for item in downlinks {
        single_file_download(item, package_name, output_dir).await?;
    }

    Ok(())
}

/// Downloads APK files from APKMirror based on the provided DownloadApkMirror.
/// Creates the output directory if it doesn't exist.
/// Downloads each file to the output directory, using the package name, version, arch, dpi
/// and extension to construct a filename.
/// Shows a progress bar while downloading.
///
/// # Arguments
///
/// * `item` - The DownloadApkMirror struct to download.
/// * `package_name` - The package name of the APK file.
/// * `output_dir` - The output directory to download the APK files to.
///
/// # Returns
///
/// A `Result` containing `()` or an `Error` if the download fails.
///
/// # Example
///
/// ```rust
/// use downapk::apkmirror::{ApkMirror, single_file_download};
///
/// #[tokio::main]
/// async fn main() {
///    let apk_mirror = ApkMirror::new().await;
///   let downloads = apk_mirror._download_by_arch("https://www.apkmirror.com/apk/instagram/instagram-lite/instagram-lite-390-0-0-9-116-release/", Some("arm64-v8a")).await.unwrap();
///  single_file_download(&downloads[0], "com.instagram.lite", "downloads").await.unwrap();
/// }
/// ```
pub async fn single_file_download(
    item: &DownloadApkMirror,
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

    let url = &item.download_link;
    let version = &item.version;
    let arch = &item.arch;
    let dpi = &item.screen_dpi;
    let extension = match item.type_.as_str() {
        "APK" => "apk",
        "BUNDLE" => "apkm",
        ext => panic!("Got an unknown apk type: {}", ext),
    };

    let mut res = reqwest::get(url).await?;
    let total_size = res.content_length().unwrap_or_default();

    let pb = match total_size {
        0 => ProgressBar::new(100),
        _ => ProgressBar::new(total_size),
    };
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

    Ok(())
}

#[cfg(test)]
mod tests;
