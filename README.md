ShadyURL-Rust
-------------
[![License: CC0-1.0](https://licensebuttons.net/l/zero/1.0/80x15.png)](http://creativecommons.org/publicdomain/zero/1.0/)
[![Stale issues and pull requests](https://github.com/Elizafox/shadyurl-rust/actions/workflows/stale.yml/badge.svg)](https://github.com/Elizafox/shadyurl-rust/actions/workflows/stale.yml)
[![Build](https://github.com/Elizafox/shadyurl-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/Elizafox/shadyurl-rust/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/Elizafox/shadyurl-rust/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/Elizafox/shadyurl-rust/actions/workflows/rust-clippy.yml)
[![DevSkim](https://github.com/Elizafox/shadyurl-rust/actions/workflows/devskim.yml/badge.svg)](https://github.com/Elizafox/shadyurl-rust/actions/workflows/devskim.yml)

This is like [ShadyURL](https://github.com/Elizafox/ShadyURL) but written in Rust and better done.

Copy the env_example file to .env, edit it, run `sea-orm-cli migrate -d <database URL>`, and you're off to the races.

Contributing
============
By contributing, you agree to disclaim all copyright to your code and release it under the [CC0 waiver](https://creativecommons.org/share-your-work/public-domain/cc0/). You also agree you have all the relevant rights to do so.

You must run the following before submitting a pull request:
* `cargo fmt`
* `cargo clippy` and fix any relevant issues (or `cargo clippy --fix`)
* If any dependencies are updated, `spdx-sbom-generator` or similar to update the SBOM
