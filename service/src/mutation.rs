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

// Database mutation operations for ShadyURL

use ipnetwork::{IpNetwork, Ipv6Network};
use sea_orm::*;

use ::entity::{cidr_ban, prelude::*, url, url_filter, user};

use crate::Query;

pub struct Mutation;

impl Mutation {
    // Create a CIDR ban given a network, reason, and user
    pub async fn create_cidr_ban(
        db: &DbConn,
        network: IpNetwork,
        reason: Option<String>,
        user: &user::Model,
    ) -> Result<cidr_ban::ActiveModel, DbErr> {
        let (start, end) = match network {
            IpNetwork::V4(n) => {
                let addr = n.network().to_ipv6_mapped();
                let prefix = n.prefix() + 96;
                let network =
                    // Should not fail, as this is coming from a valid IPv4 network
                    Ipv6Network::new(addr, prefix).expect("Could not create IPv6 network");
                (network.network(), network.broadcast())
            }
            IpNetwork::V6(n) => (n.network(), n.broadcast()),
        };

        cidr_ban::ActiveModel {
            range_begin: ActiveValue::Set(start.octets().to_vec()),
            range_end: ActiveValue::Set(end.octets().to_vec()),
            reason: ActiveValue::Set(reason),
            user_created_id: Set(Some(user.id)),
            ..Default::default()
        }
        .save(db)
        .await
    }

    // Create a user given a username and password hash
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

    // Create a URL given a url, shady "filename", and IP
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

    // Create a URL filter given a filter string, an optional reason, and a user.
    pub async fn create_url_filter(
        db: &DbConn,
        filter: String,
        reason: Option<String>,
        user: &user::Model,
    ) -> Result<url_filter::ActiveModel, DbErr> {
        url_filter::ActiveModel {
            filter: Set(filter),
            reason: Set(reason),
            user_created_id: Set(Some(user.id)),
            ..Default::default()
        }
        .save(db)
        .await
    }

    // Change a user password given a username and password hash.
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

    // Delete a CIDR ban by ID.
    pub async fn delete_cidr_ban(db: &DbConn, id: i64) -> Result<DeleteResult, DbErr> {
        CidrBan::delete_by_id(id).exec(db).await
    }

    // Delete a user by ID.
    pub async fn delete_user(db: &DbConn, id: i64) -> Result<DeleteResult, DbErr> {
        User::delete_by_id(id).exec(db).await
    }

    // Delete a URL by ID.
    pub async fn delete_url(db: &DbConn, id: i64) -> Result<DeleteResult, DbErr> {
        Url::delete_by_id(id).exec(db).await
    }

    // Delete a URL filter by ID.
    pub async fn delete_url_filter(db: &DbConn, id: i64) -> Result<DeleteResult, DbErr> {
        UrlFilter::delete_by_id(id).exec(db).await
    }
}
