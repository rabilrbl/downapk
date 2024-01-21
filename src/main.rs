mod apkmirror;
mod errors;
mod utils;

use apkmirror::{multiple_file_download, single_file_download, ApkMirror, ApkType};
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum DownloadOption {
    One,
    All,
}

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
    /// Default: Both
    #[arg(short('t'), long)]
    apk_type: Option<ApkType>,

    /// Optional: Screen DPI
    /// Possible values: nodpi, 120-320, ..., all
    #[arg(long, default_value_t = String::from("all"))]
    dpi: String,

    /// Optional: Search Index to download
    /// Possible values: 1, 2, 3, ...
    /// Default: None. User will be prompted to choose an index
    #[arg(short, long)]
    search_index: Option<usize>,

    /// Optional: Whether to download all apks or one from final download page
    /// Default: None. User will be prompted to choose an index
    #[arg(short, long)]
    download_option: Option<DownloadOption>,

    /// If download option is `one` then this is the index of the apk to download
    /// Possible values: 1, 2, 3, ...
    /// Default: None. User will be prompted to choose an index
    #[arg(short('i'), long)]
    download_index: Option<usize>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let apkmirror = ApkMirror::new()
        .await
        .unwrap_or_else(|err| panic!("Error while creating ApkMirror instance. Err: {}", err));

    let package_id = args.package_id;
    let output_dir = args.output_dir;
    let arch = match args.arch.as_str() {
        "all" | "ALL" => None,
        _ => Some(args.arch.as_str()),
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

    let choice = args.search_index.unwrap_or_else(|| {
        // print all results.i.link with number
        for (i, result) in results.iter().enumerate() {
            println!(
                "{}. {} {} {}",
                i + 1,
                result.title,
                result.uploaded,
                result.file_size
            );
        }
        read_input("Choose a number from above to download:")
    });

    // make sure choice is within range of results
    if choice == 0 || choice > results.len() {
        panic!(
            "Invalid search index. Choose a number from 1 to {}",
            results.len()
        );
    }
    let download_url = results[choice - 1].link.clone();
    let download_result = apkmirror
        .download_by_specifics(&download_url, args.apk_type, arch, dpi)
        .await
        .unwrap_or_else(|err| {
            panic!("Error while calling download_by_specifics. Err {}", err);
        });

    let choice: usize = match download_result.len() {
        0 => {
            panic!("No apk files found for download. Retry again after some time");
        }
        1 => 2,
        _ => match args.download_option {
            Some(ref option) => match option {
                DownloadOption::All => 2,
                DownloadOption::One => 1,
            },
            None => {
                println!("There are multiple apk files available for download");
                println!("1. Download one specific file");
                println!("2. Download all files");
                read_input("Choose a number from above:")
            }
        },
    };

    println!();
    match choice {
        1 => {
            let choice = args.download_index.unwrap_or_else(|| {
                for (i, result) in download_result.iter().enumerate() {
                    println!(
                        "{}. {} {} {} {} {}",
                        i + 1,
                        result.version,
                        result.apk_type,
                        result.arch,
                        result.screen_dpi,
                        result.min_version
                    );
                }
                read_input("Choose a number from above to download:")
            });

            // make sure choice is within range of download_result
            if choice > download_result.len() {
                panic!(
                    "Invalid download index. Choose a number from 1 to {}",
                    download_result.len()
                );
            }

            match single_file_download(&download_result[choice - 1], &package_id, &output_dir).await
            {
                Ok(_) => println!("Downloaded successfully"),
                Err(e) => panic!("Error while downloading. Err: {}", e),
            }
        }
        2 => match multiple_file_download(&download_result, &package_id, &output_dir).await {
            Ok(_) => println!("Downloaded successfully"),
            Err(e) => panic!("Error while downloading. Err: {}", e),
        },
        _ => println!("Invalid choice"),
    }
}

fn read_input(msg: &str) -> usize {
    println!("{}", msg);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().parse().unwrap_or_else(|err| {
        panic!("Error parsing input: {}", err);
    })
}
