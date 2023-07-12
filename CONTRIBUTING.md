Contributing
------------
Contributions are welcome to ShadyURL-Rust! However, you must agree to these stipulations and follow these guidelines to contribute.

As a matter of my (Elizabeth Myers) own personal opinion, I do not personally consider these too onerous, or even unusual. I'm merely being explicit about them.

Code of conduct
===============
By contributing, you agree to abide by the [Contributor Covenant](/CODE_OF_CONDUCT.md). We aim to foster an inclusive and welcoming environment for all, provided they are able to follow the rules and make others feel welcome as well.

Failure to abide by the covenant may result in expulsion from the project.

Copyright
=========
By contributing, you agree to disclaim all copyright to your code and release it under the [CC0 waiver](https://creativecommons.org/share-your-work/public-domain/cc0/). You also agree you have all the relevant rights to do so.

You must ensure the following before submitting a pull request:
* That all new files have a header similar to the [header](#header) shown here.
* Ensure all your commits are signed off (`git commit --signoff`) and you comply with the [Developer's Certificate of Origin](/DCO.txt)
* Ensure all dependencies linked are compatible with CC0
* Assert you comply with our [patent policy](#patents)

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

Patents
=======
The CC0 waiver specifically protects your patent rights. Therefore, by contributing, *per se*, you do not waive such rights.

However, for your contributions to be accepted, you have two options regarding your patents:
- You must assert you have no patent rights in the code, and to the best of your knowledge, your contributions are not covered by active patents.
- You must agree in writing not to enforce your patents against any users of ShadyURL-Rust. Such an agreement must be signed, irrevocable, and made public. Relevant patents must be mentioned in said agreement.

Developer Certificate of Origin
===============================
You must acknowledge and affirm you are in compliance with the [Developer's Ceritifcate of Origin](/DCO.txt). All commits must have a `Signed-off-by` line. Use `git commit --signoff` to sign-off your commits.

Style
=====
Before you commit, you must do the following:

* Run `cargo fmt`
* Run `cargo clippy` and fix any relevant issues (or `cargo clippy --fix`)

Compliance
==========
Failure to abide by the code of conduct, patent, and copyright policies will result in expulsion from the project. The reasoning is simple: those unwilling to abide by our policies cannot be trusted to contribute further.

Style violations will not be treated as severely, but you might get admonished. 😜
