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

You must ensure the following before submitting a pull request:
* That all new files have a header similar to the [header](#header) shown here.
* Ensure all your commits are signed off (`git commit --signoff`) and you comply with the [Developer's Certificate of Origin](/DCO.txt)
* That you have run `cargo fmt`
* That you have run `cargo clippy` and fixed any relevant issues (or `cargo clippy --fix`)
* If any dependencies are updated, [`spdx-sbom-generator`](https://github.com/opensbom-generator/spdx-sbom-generator) or similar to update the SBOM

### Header
All new files must include a header similar to this:

```rust
/* SPDX-License-Identifier: CC0-1.0
 *
 * <path to file>
 *
 * This file is a component of ShadyURL by Elizabeth Myers.
 * The author of this file is <author> and has agreed to the below waiver.
 *
 * To the extent possible under law, the person who associated CC0 with
 * ShadyURL has waived all copyright and related or neighboring rights
 * to ShadyURL.
 *
 * You should have received a copy of the CC0 legalcode along with this
 * work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

```
