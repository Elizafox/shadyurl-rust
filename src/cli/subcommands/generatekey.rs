/* SPDX-License-Identifier: CC0-1.0
 *
 * src/cli/subcommands/generatekey.rs
 *
 * This file is a component of ShadyURL by Elizabeth Myers.
 *
 * To the extent possible under law, the person who associated CC0 with
 * ShadyURL has waived all copyright and related or neighboring rights
 * to ShadyURL.
 *
 * You should have received a copy of the CC0 legalcode along with this
 * work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

use base64::prelude::*;
use rand::prelude::*;

use crate::cli::subcommands::CliSubcommand;
use crate::env::{Key, Vars};

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Rand(#[from] rand::Error),
}

pub struct GenerateKeySubcommand;

#[async_trait::async_trait]
impl CliSubcommand for GenerateKeySubcommand {
    type Error = CliError;
    type PromptUserData = ();
    type CommandData = ();

    fn proc_title() -> String {
        "shadyurl-rust [generate-key]".to_string()
    }

    fn prompt_user() -> Result<Self::PromptUserData, Self::Error> {
        Ok(())
    }

    async fn run(
        _: Vars,
        _: Self::PromptUserData,
        (): &Self::CommandData,
    ) -> Result<(), Self::Error> {
        let mut rng = thread_rng();

        let mut enc: Key = [0u8; 64];

        // Random key material will do just fine
        rng.try_fill_bytes(&mut enc)?;

        let enc_b64 = BASE64_STANDARD.encode(enc.as_ref());

        println!("CSRF_KEY=\"{enc_b64}\"");
        Ok(())
    }
}
