# DownAPK

Program to download APKs of given Android package

# Installation

## Pre-built binaries

Download the latest binary release from [here](https://github.com/rabilrbl/downapk/releases/latest).

```shell
# Unix-like (Linux, macOS, Android)
chmod +x downapk-linux
./downapk-linux --help

# Windows
downapk-windows.exe --help
```

## Cargo

```
cargo install downapk
downapk --help
```

## Build from source

```
cargo build --release
./target/release/downapk --help
```

## Usage

```shell
downapk [OPTIONS] --package-id <PACKAGE_ID>
```

### Options

| Option |   Description | Default Value |
| --- | --- | --- |
| `-p, --package-id <PACKAGE_ID>`     | Android package ID | -             |
| `-o, --output-dir <OUTPUT_DIR>`     | Optional: Output file name | downloads     |
| `-a, --arch <ARCH>`                 | Optional: Architecture. Possible values: arm64-v8a, armeabi-v7a, x86, x86_64, universal | all  |
| `-v, --version-code <VERSION_CODE>` | Optional: Version code. Possible values: latest, x.x.x (e.g. 1.0.0 | latest |
| `-t, --type <TYPE>`                 | Optional: Type of APK. Possible values: bundle, apk | all   |
| `-d, --dpi <DPI>`                   | Optional: Screen DPI. Possible values: nodpi, 120-320, ..., | all           |
| `-h, --help`                        | Print help | -             |
| `-V, --version`                     | Print version | -             |

### Examples

```shell
# Download all APKs of package com.google.android.youtube of universal architecture and latest version with nodpi
downapk -p com.google.android.youtube -t apk -a universal -d nodpi
```

```shell
# Download all APKs of package com.google.android.youtube of universal architecture and version 14.21.54 with nodpi
downapk -p com.google.android.youtube -t apk -a universal -d nodpi -v 14.21.54
```

## License

MIT License. See [LICENSE](LICENSE) file for details.

## Author

[Mohammed Rabil](https://github.com/rabilrbl)

## Contributors

[![Contributors](https://contributors-img.web.app/image?repo=rabilrbl/downapk)](https://github.com/rabilrbl/downapk/graphs/contributors)
