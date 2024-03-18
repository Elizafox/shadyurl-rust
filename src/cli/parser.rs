/* SPDX-License-Identifier: CC0-1.0
 *
 * src/cli/parser.rs
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

use clap::{Args, Parser, Subcommand};

use crate::cli::subcommands::{
    AddUserSubcommand, ChangePasswordSubcommand, CliSubcommand, DeleteUserSubcommand,
    GenerateKeysSubcommand, RunSubcommand,
};

// Simple things
#[derive(Debug, Clone, Args)]
pub struct UsernameArgument {
    pub username: String,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Run,
    AddUser(UsernameArgument),
    DeleteUser(UsernameArgument),
    ChangePassword(UsernameArgument),
    GenerateKeys,
}

pub async fn run_command() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::AddUser(data)) => {
            AddUserSubcommand::run_command(data).await?;
            Ok(())
        }
        Some(Commands::DeleteUser(data)) => {
            DeleteUserSubcommand::run_command(data).await?;
            Ok(())
        }
        Some(Commands::ChangePassword(data)) => {
            ChangePasswordSubcommand::run_command(data).await?;
            Ok(())
        }
        Some(Commands::GenerateKeys) => {
            GenerateKeysSubcommand::run_command(&()).await?;
            Ok(())
        }
        Some(Commands::Run) | None => {
            RunSubcommand::run_command(&()).await?;
            Ok(())
        }
    }
}
