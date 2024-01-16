mod downapk;

use downapk::ApkMirror;

#[tokio::main]
async fn main() {
    let downapk = ApkMirror::new().await;
    let result = downapk.search("com.google.android.youtube").await;

    match result {
        Ok(result) => {
            let download_url = result[0]["link"].as_str().unwrap();
            let download_result = downapk.download(download_url).await;

            match download_result {
                Ok(download_result) => {
                    println!("{}", download_result);
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
        
        Err(err) => {
            println!("{}", err);
        }
    }
}
