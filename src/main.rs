mod downapk;

use downapk::ApkMirror;

#[tokio::main]
async fn main() {
    let apkmirror = ApkMirror::new().await;

    // take input from user
    println!("Enter package name");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let result = apkmirror.search(&input).await;

    match result {
        Ok(result) => {
            let download_url = result[0]["link"].as_str().unwrap();
            let download_result = apkmirror.download(download_url).await;

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
