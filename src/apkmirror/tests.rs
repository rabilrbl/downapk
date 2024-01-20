use super::*;

#[tokio::test]
async fn test_search() {
    let downloader = ApkMirror::new().await;
    let search_query = "com.google.android.youtube";
    let version = "19.02.34";
    let result = downloader.search_by_version(search_query, version).await;
    assert!(result.is_ok());
    let extracted_links = result.unwrap();
    assert!(!extracted_links.is_empty());
    for item in extracted_links {
        assert!(!item.title.is_empty());
        assert!(!item.link.is_empty());
        assert!(!item.version.is_empty());
        assert_eq!(item.version, version.to_string());
    }
}

#[tokio::test]
async fn test_extract_root_links() {
    let downloader = ApkMirror::new().await;
    let url = "https://www.apkmirror.com/?post_type=app_release&searchtype=apk&s=com.google.android.youtube";
    let result = downloader.extract_root_links(url, None).await;
    assert!(result.is_ok());
    let extracted_links = result.unwrap();
    assert!(!extracted_links.is_empty());
    for item in extracted_links {
        assert!(!item.title.is_empty());
        assert!(!item.link.is_empty());
        assert!(!item.version.is_empty());
    }
}

#[tokio::test]
async fn test_download() {
    let downloader = ApkMirror::new().await;
    let url = "https://www.apkmirror.com/apk/instagram/instagram-lite/instagram-lite-390-0-0-9-116-release/";
    let arch = "arm64-v8a";
    let type_ = "APK";
    let dpi = "nodpi";
    let result = downloader
        .download_by_specifics(url, Some(type_), Some(arch), Some(dpi))
        .await;
    assert!(result.is_ok());
    let download_apkmirror_result =
        result.unwrap_or_else(|e| panic!("Error while unwrapping result. Err: {}", e));
    assert!(!download_apkmirror_result.is_empty());
    for item in &download_apkmirror_result {
        assert!(!item.version.is_empty());
        assert!(!item.download_link.is_empty());
        assert!(!item.type_.is_empty());
        assert!(!item.arch.is_empty());
        assert!(!item.min_version.is_empty());
        assert!(!item.screen_dpi.is_empty());
        assert_eq!(item.type_, type_.to_string());
        assert_eq!(item.arch, arch.to_string());
        assert_eq!(item.screen_dpi, dpi.to_string());
    }

    match single_file_download(
        &download_apkmirror_result[0],
        "com.instagram.lite",
        "downloads",
    )
    .await
    {
        Ok(_) => {
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
