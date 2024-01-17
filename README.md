# DownAPK

Program to download APKs of given Android package

## Usage

```
Usage: downapk [OPTIONS] --package-id <PACKAGE_ID>

Options:
  -p, --package-id <PACKAGE_ID>      Android package ID
  -o, --output-dir <OUTPUT_DIR>      Optional: Output file name [default: downloads]
  -a, --arch <ARCH>                  Optional: Architecture Possible values: arm64-v8a, armeabi-v7a, x86, x86_64, universal, all [default: all]
  -v, --version-code <VERSION_CODE>  Optional: Version code Possible values: latest, x.x.x (e.g. 1.0.0) [default: latest]
  -t, --type <TYPE>                  Optional: Type of APK Possible values: bundle, apk, all [default: all]
  -d, --dpi <DPI>                    Optional: Screen DPI Possible values: nodpi, 120-320, ..., all [default: all]
  -h, --help                         Print help
  -V, --version                      Print version
```

## License

MIT License. See [LICENSE](LICENSE) file for details.

## Author

[Mohammed Rabil](https://github.com/rabilrbl)

## Contributors

[![Contributors](https://contributors-img.web.app/image?repo=rabilrbl/downapk)](https://github.com/rabilrbl/downapk/graphs/contributors)
