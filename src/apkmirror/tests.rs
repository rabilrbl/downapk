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
