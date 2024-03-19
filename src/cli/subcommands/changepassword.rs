/* SPDX-License-Identifier: CC0-1.0
 *
 * src/cli/subcommands/changepassword.rs
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

use password_auth::generate_hash;
use rpassword::prompt_password;

use crate::{
    cli::subcommands::{CliSubcommand, UsernameArgument},
    env::{EnvError, Vars},
};

use service::{Database, Mutation};

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Env(#[from] EnvError),

    #[error("Passwords did not match")]
    PasswordMismatch,
}

pub struct ChangePasswordData {
    password: String,
}

pub struct ChangePasswordSubcommand;

#[async_trait::async_trait]
impl CliSubcommand for ChangePasswordSubcommand {
    type Error = CliError;
    type PromptUserData = ChangePasswordData;
    type CommandData = UsernameArgument;

    fn proc_title() -> String {
        "shadyurl-rust [change-password]".to_string()
    }

    fn prompt_user() -> Result<Self::PromptUserData, Self::Error> {
        let mut password = prompt_password("Password:")?;
        if password != prompt_password("Repeat password:")? {
            return Err(CliError::PasswordMismatch);
        }

        password = generate_hash(password);

        Ok(Self::PromptUserData { password })
    }

    async fn run(
        env: Vars,
        prompt: Self::PromptUserData,
        data: &Self::CommandData,
    ) -> Result<(), Self::Error> {
        let db = Database::get(&env.database_url).await?;
        Mutation::change_user_password(&db, &data.username, &prompt.password).await?;
        Ok(())
    }
}
