mod downapk;

use downapk::HttpRequester;

#[tokio::main]
async fn main() {
    let downapk = HttpRequester::new();
    let result = downapk.index().await;

    match result {
        Ok(value) => println!("{:?}", value),
        Err(error) => eprintln!("Error: {}", error),
    }
}
