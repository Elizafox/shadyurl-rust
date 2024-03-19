/* SPDX-License-Identifier: CC0-1.0

* src/cli/subcommands.rs
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

// Subcommand related stuff

mod adduser;
mod changepassword;
mod deleteuser;
mod generatekeys;
mod run;

use proctitle::set_title;

use crate::env::{EnvError, Vars};

// Re-exported
pub(crate) use crate::cli::parser::UsernameArgument;

#[async_trait::async_trait]
pub trait CliSubcommand {
    type Error;
    type PromptUserData: Send + Sync;
    type CommandData: Send + Sync;

    fn proc_title() -> String {
        "shadyurl-rust [command]".to_string()
    }

    fn load_env() -> Result<Vars, EnvError> {
        Vars::load_env()
    }

    fn prompt_user() -> Result<Self::PromptUserData, Self::Error>;

    fn check_cli_args(_: Self::CommandData) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn run_command(data: &Self::CommandData) -> Result<(), Self::Error> {
        set_title(Self::proc_title());
        let env = Self::load_env().expect("Could not load env");
        let prompt = Self::prompt_user()?;
        Self::run(env, prompt, data).await
    }

    async fn run(
        env: Vars,
        prompt: Self::PromptUserData,
        data: &Self::CommandData,
    ) -> Result<(), Self::Error>;
}

pub use adduser::AddUserSubcommand;
pub use changepassword::ChangePasswordSubcommand;
pub use deleteuser::DeleteUserSubcommand;
pub use generatekeys::GenerateKeysSubcommand;
pub use run::RunSubcommand;
