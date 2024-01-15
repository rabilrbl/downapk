mod downapk;

use downapk::HttpRequester;

#[tokio::main]
async fn main() {
    let downapk = HttpRequester::new();
    let result = downapk.get_app_search("com.google.android.youtube", 13).await;

    match result {
        Ok(value) => println!("{:?}", value),
        Err(error) => eprintln!("Error: {}", error),
    }
}
