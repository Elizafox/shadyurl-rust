/* SPDX-License-Identifier: CC0-1.0
 *
 * service/src/mutation.rs
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

use ::entity::{url, user};
use sea_orm::*;

use crate::Query;

pub struct Mutation;

impl Mutation {
    pub async fn create_user(
        db: &DbConn,
        username: &str,
        password_hash: &str,
    ) -> Result<user::ActiveModel, DbErr> {
        user::ActiveModel {
            username: Set(username.to_owned()),
            password_hash: Set(password_hash.to_owned()),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn change_user_password(
        db: &DbConn,
        username: &str,
        password_hash: &str,
    ) -> Result<user::ActiveModel, DbErr> {
        let mut user: user::ActiveModel = Query::find_user_by_username(db, username)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user.password_hash = Set(password_hash.to_string());
        user.update(db).await.map(Into::into)
    }

    pub async fn delete_user(db: &DbConn, id: i64) -> Result<DeleteResult, DbErr> {
        let user: user::ActiveModel = Query::find_user_by_id(db, id)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user.delete(db).await
    }

    pub async fn create_url(
        db: &DbConn,
        url: &str,
        shady: &str,
        ip: Option<String>,
    ) -> Result<url::ActiveModel, DbErr> {
        url::ActiveModel {
            url: Set(url.to_owned()),
            shady: Set(shady.to_owned()),
            ip: Set(ip),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn delete_url(db: &DbConn, id: i64) -> Result<DeleteResult, DbErr> {
        let url: url::ActiveModel = Query::find_url_by_id(db, id)
            .await?
            .ok_or(DbErr::Custom("Cannot find url.".to_owned()))
            .map(Into::into)?;

        url.delete(db).await
    }
}
