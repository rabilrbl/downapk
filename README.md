# DownAPK

Program to download APKs of given Android package

## Usage

```
Usage: downapk [OPTIONS] --package-id <PACKAGE_ID>

Options:
  -p, --package-id <PACKAGE_ID>
          Android package ID
  -o, --output-file-name <OUTPUT_FILE_NAME>
          Optional: Output file name [default: output.apk]
  -a, --arch <ARCH>
          Optional: Architecture Possible values: arm64-v8a, armeabi-v7a, x86, x86_64, all [default: all]
  -v, --version-code <VERSION_CODE>
          Optional: Version code Possible values: latest, x.x.x (e.g. 1.0.0) [default: latest]
  -t, --type <TYPE>
          Optional: Type of APK Possible values: bundle, apk, all [default: all]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples

```shell
downapk --package-id com.google.android.youtube
downapk --package-id com.google.android.youtube --output-file-name youtube.apk
downapk --package-id com.google.android.youtube --arch arm64-v8a
downapk --package-id com.google.android.youtube --version-code 1.0.0
downapk --package-id com.google.android.youtube --type apk
```

# License

MIT License. See [LICENSE](LICENSE) file for details.

# Author

[Mohammed Rabil](https://github.com/rabilrbl)

# Contributors

[![Contributors](https://contributors-img.web.app/image?repo=rabilrbl/downapk)](https://github.com/rabilrbl/downapk/graphs/contributors)
