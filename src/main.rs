/* SPDX-License-Identifier: CC0-1.0
 *
 * src/main.rs
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

#![warn(unused_extern_crates)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

mod auth;
mod bancache;
mod csrf;
mod env;
mod err;
mod generate;
mod state;
mod util;
mod validators;
mod web;

use std::io::{prelude::*, stdin, stdout};

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use mimalloc::MiMalloc;
use password_auth::generate_hash;
use proctitle::set_title;
use rpassword::prompt_password;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{env::Vars, web::App};

use migration::{Migrator, MigratorTrait};
use service::{Database, Mutation, Query};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run,
    AddUser { username: String },
    DeleteUser { username: String },
    ChangePassword { username: String },
}

#[derive(Debug, thiserror::Error)]
enum UserError {
    #[error("Could not alter user {}: {}", .0, .1)]
    Change(String, String),

    #[error("Could not create user {}: {}", .0, .1)]
    Add(String, String),

    #[error("Could not delete user {}: {}", .0, .1)]
    Delete(String, String),
}

async fn add_user_cli(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env = Vars::load_env()?;
    let db = Database::get(&env.database_url).await?;

    let mut password = prompt_password("Password:")?;
    if password != prompt_password("Repeat password:")? {
        eprintln!("Passwords do not match");
        return Err(Box::new(UserError::Add(
            username.to_string(),
            "Passwords did not match".to_string(),
        )));
    }

    password = generate_hash(password);

    Mutation::create_user(&db, username, &password).await?;

    Ok(())
}

async fn delete_user_cli(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env = Vars::load_env()?;
    let db = Database::get(&env.database_url).await?;

    let mut response = String::new();
    loop {
        print!("Are you SURE you want to delete user {username}? [yes/no] ");
        stdout().flush()?;
        stdin().lock().read_line(&mut response)?;
        response = response.trim_end().to_ascii_lowercase().to_string();
        match response.as_str() {
            "no" | "n" => {
                return Err(Box::new(UserError::Delete(
                    username.to_string(),
                    "Aborted".to_string(),
                )))
            }
            "yes" => break,
            _ => {
                response.clear();
                println!("Please type yes or no.");
            }
        }
    }

    let user = Query::find_user_by_username(&db, username)
        .await?
        .ok_or_else(|| UserError::Delete(username.to_string(), "Username not found".to_string()))?;

    Mutation::delete_user(&db, user.id).await?;

    Ok(())
}

async fn change_password_cli(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env = Vars::load_env()?;
    let db = Database::get(&env.database_url).await?;

    let mut password = prompt_password("Password:")?;
    if password != prompt_password("Repeat password:")? {
        eprintln!("Passwords do not match");
        return Err(Box::new(UserError::Change(
            username.to_string(),
            "Passwords did not match".to_string(),
        )));
    }

    password = generate_hash(password);

    Mutation::change_user_password(&db, username, &password).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Important to do this first, as our logging config may be in .env
    dotenv()?;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::AddUser { username }) => {
            set_title("shadyurl-rust <add-user>");
            add_user_cli(username).await?;
            println!("Success! User {username} added");
            return Ok(());
        }
        Some(Commands::DeleteUser { username }) => {
            set_title("shadyurl-rust <delete-user>");
            delete_user_cli(username).await?;
            println!("Success! User {username} deleted");
            return Ok(());
        }
        Some(Commands::ChangePassword { username }) => {
            set_title("shadyurl-rust <change-password>");
            change_password_cli(username).await?;
            println!("Success! Password for {username} changed");
            return Ok(());
        }
        Some(Commands::Run) | None => set_title("shadyurl-rust [running]"),
    }

    App::new().await?.serve().await
}
