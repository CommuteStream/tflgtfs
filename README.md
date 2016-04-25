# Transport For London GTFS Exporter

This simple Rust CLI allows you to fetch data from the
[Tfl Unified API][tfl-api] and transform it to [GTFS][gtfs].

[![Build Status](https://travis-ci.org/CommuteStream/tflgtfs.svg?branch=master)](https://travis-ci.org/CommuteStream/tflgtfs)
[![Clippy Linting Result](https://clippy.bashy.io/github/CommuteStream/tflgtfs/master/badge.svg)](https://clippy.bashy.io/github/CommuteStream/tflgtfs/change-to-serde/log)


## Install

Clone [the repository][tfl-cli] and compile:

```sh
cargo build --release
```

*WARNING*: If you compile under OSX 10.11 you might need to specify the
OpenSSL include path.  For example, having OpenSSL installed via Homebrew,
the command is:

```sh
OPENSSL_INCLUDE_DIR=/usr/local/opt/openssl/include cargo build --release
```

You will find the binary in `./target/release/`.


## Usage

Check the help `./target/release/tflgtfs help` for details.

In short, you can fetch Tfl lines with the `fetch-lines` command and transform
the cached values with the `transform gtfs` command.

You can do it in one shot via:

```sh
./target/release/tflgtfs fetch-lines --format gtfs
```

You will find the resulting GTFS files inside `./gtfs`.


## Development

When developing on nightly build it using the following command to actually
benefit from linting and Serde macro:

```
cargo build --features nightly --no-default-features
```


## License

See [License](./LICENSE).


[tfl-cli]: https://github.com/CommuteStream/tflgtfs/
[tfl-api]: https://api.tfl.gov.uk/
[gtfs]: https://developers.google.com/transit/gtfs/
[cargo-clippy]: https://crates.io/crates/cargo-clippy

![alt travis-ci](https://travis-ci.org/CommuteStream/tflgtfs.svg?branch=master)
