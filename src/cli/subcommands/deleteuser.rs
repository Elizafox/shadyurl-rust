/* SPDX-License-Identifier: CC0-1.0
 *
 * src/cli/subcommands/deleteuser.rs
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

use std::io::{prelude::*, stdin, stdout};

use crate::{
    cli::{parser::UsernameArgument, subcommands::CliSubcommand},
    env::{EnvError, Vars},
};

use service::{Database, Mutation, Query};

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Env(#[from] EnvError),

    #[error("Aborted")]
    Aborted,

    #[error("User not found")]
    NotFound,
}

pub struct DeleteUserSubcommand;

#[async_trait::async_trait]
impl CliSubcommand for DeleteUserSubcommand {
    type Error = CliError;
    type PromptUserData = ();
    type CommandData = UsernameArgument;

    fn proc_title() -> String {
        "shadyurl-rust [delete-user]".to_string()
    }

    fn prompt_user() -> Result<Self::PromptUserData, Self::Error> {
        let mut response = String::new();
        loop {
            print!("Are you SURE you want to delete this user? [yes/no] ");
            stdout().flush()?;
            stdin().lock().read_line(&mut response)?;
            response = response.trim_end().to_ascii_lowercase().to_string();
            match response.as_str() {
                "no" | "n" => return Err(CliError::Aborted),
                "yes" => break,
                _ => {
                    response.clear();
                    println!("Please type yes or no.");
                }
            }
        }

        Ok(())
    }

    async fn run(
        env: Vars,
        _: Self::PromptUserData,
        data: &Self::CommandData,
    ) -> Result<(), Self::Error> {
        let db = Database::get(&env.database_url).await?;
        let user = Query::find_user_by_username(&db, &data.username)
            .await?
            .ok_or(CliError::NotFound)?;

        Mutation::delete_user(&db, user.id).await?;

        Ok(())
    }
}
