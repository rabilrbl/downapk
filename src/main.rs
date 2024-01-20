mod apkmirror;
mod utils;

use apkmirror::{download_file, ApkMirror};
use clap::Parser;

/// Program to download APKs of given Android package ID
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Android package ID
    #[arg(short, long)]
    package_id: String,

    /// Optional: Output file name
    #[arg(short, long, default_value_t = String::from("downloads"))]
    output_dir: String,

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

    /// Optional: Screen DPI
    /// Possible values: nodpi, 120-320, ..., all
    #[arg(short, long, default_value_t = String::from("all"))]
    dpi: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let apkmirror = ApkMirror::new().await;

    let package_id = args.package_id;
    let output_dir = args.output_dir;
    let arch = match args.arch.as_str() {
        "all" | "ALL" => None,
        _ => Some(args.arch.as_str()),
    };
    let type_: Option<&str> = match args.type_.as_str() {
        "bundle" | "BUNDLE" | "split" => Some("BUNDLE"),
        "apk" | "APK" => Some("APK"),
        _ => None,
    };
    let dpi = match args.dpi.as_str() {
        "all" | "ALL" => None,
        _ => Some(args.dpi.as_str()),
    };

    let version_code = args.version_code;
    let results = match version_code.as_str() {
        "latest" => {
            let result = apkmirror.search(&package_id).await;
            result.unwrap()
        }

        _ => {
            let result = apkmirror
                .search_by_version(&package_id, &version_code)
                .await;
            result.unwrap()
        }
    };

    for result in results {
        let download_url = result.link.as_str();
        let download_result = apkmirror
            .download_by_specifics(download_url, type_, arch, dpi)
            .await;

        match download_result {
            Ok(download_result) => download_file(&download_result, &package_id, &output_dir)
                .await
                .unwrap_or_else(|err| {
                    panic!("Could not download file: {}", err);
                }),
            Err(err) => {
                panic!("Error: {}", err);
            }
        }
    }
}
