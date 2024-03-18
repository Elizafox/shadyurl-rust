/* SPDX-License-Identifier: CC0-1.0
 *
 * src/cli/subcommands/run.rs
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

use crate::{
    cli::subcommands::CliSubcommand,
    env::Vars,
    web::{App, RuntimeError},
};

#[allow(clippy::module_name_repetitions)]
pub struct RunSubcommand;

#[async_trait::async_trait]
impl CliSubcommand for RunSubcommand {
    type Error = RuntimeError;
    type PromptUserData = ();
    type CommandData = ();

    fn prompt_user() -> Result<Self::PromptUserData, Self::Error> {
        Ok(())
    }

    async fn run(
        _: Vars,
        _: Self::PromptUserData,
        (): &Self::CommandData,
    ) -> Result<(), Self::Error> {
        App::new().await?.serve().await
    }
}
