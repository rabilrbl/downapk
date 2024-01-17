mod downapk;

use clap::Parser;
use downapk::ApkMirror;

/// Program to download APKs of given Android package ID
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Android package ID
    #[arg(short, long)]
    package_id: String,

    /// Optional: Output file name
    #[arg(short, long, default_value_t = String::from("output.apk"))]
    output_file_name: String,

    /// Optional: Architecture
    /// Possible values: arm64-v8a, armeabi-v7a, x86, x86_64, universal, all
    #[arg(short, long, default_value_t = String::from("all"))]
    arch: String,

    /// Optional: Version code
    /// Possible values: latest, x.x.x (e.g. 1.0.0)
    #[arg(short, long, default_value_t = String::from("latest"))]
    version_code: String,

    /// Optional: Type of APK
    /// Possible values: bundle, apk, all
    #[arg(short, long, default_value_t = String::from("all"))]
    type_: String,
}

#[tokio::main]
async fn main() {

    let args = Args::parse();

    let apkmirror = ApkMirror::new().await;

    let package_id = args.package_id;
    // let output_file_name = args.output_file_name;
    let arch = match args.arch.as_str() {
        "all" | "ALL" => None,
        _ => Some(args.arch.as_str()),
    };
    let type_: Option<&str> = match args.type_.as_str() {
        "bundle" | "BUNDLE" | "split" => Some("BUNDLE"),
        "apk" | "APK" => Some("APK"),
        _ => None,
    };

    let version_code = args.version_code;

    match version_code.as_str() {
        "latest" => {
            let result = apkmirror.search(&package_id).await;

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

        _ => {
            let result = apkmirror.search_by_version(&package_id, &version_code).await;

            match result {
                Ok(result) => {
                    let download_url = result[0]["link"].as_str().unwrap();
                    let download_result = apkmirror.download_by_type_arch(download_url, type_, arch).await;

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
    }
}
