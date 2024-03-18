/* SPDX-License-Identifier: CC0-1.0
 *
 * service/src/query.rs
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

// Database query operations for ShadyURL.

use std::net::IpAddr;

use sea_orm::*;

use ::entity::{cidr_ban, prelude::*, url, url_filter, user};

pub struct Query;

impl Query {
    // Find a CIDR ban by ID.
    pub async fn find_cidr_ban(db: &DbConn, id: i64) -> Result<Option<cidr_ban::Model>, DbErr> {
        CidrBan::find_by_id(id).one(db).await
    }

    // Find a user by ID.
    pub async fn find_user_by_id(db: &DbConn, id: i64) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    // Find a user by username.
    pub async fn find_user_by_username(
        db: &DbConn,
        username: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
    }

    // Find a URL by its pointer.
    pub async fn find_url_by_string(db: &DbConn, url: &str) -> Result<Vec<url::Model>, DbErr> {
        Url::find().filter(url::Column::Url.eq(url)).all(db).await
    }

    // Find a URL by its shady filename.
    pub async fn find_url_by_shady_string(
        db: &DbConn,
        shady: &str,
    ) -> Result<Option<url::Model>, DbErr> {
        Url::find()
            .filter(url::Column::Shady.eq(shady))
            .one(db)
            .await
    }

    // Find a URL by its ID.
    pub async fn find_url_by_id(db: &DbConn, id: i64) -> Result<Option<url::Model>, DbErr> {
        Url::find_by_id(id).one(db).await
    }

    // Get all URL's in the database.
    // TODO: pagination?
    pub async fn fetch_all_urls(db: &DbConn) -> Result<Vec<url::Model>, DbErr> {
        Url::find().order_by_asc(url::Column::Id).all(db).await
    }
    
    // Find a URL filter by its ID.
    pub async fn find_url_filter(db: &DbConn, id: i64) -> Result<Option<url_filter::Model>, DbErr> {
        UrlFilter::find_by_id(id).one(db).await
    }


    // Get all CIDR bans in the database.
    // TODO: pagination?
    pub async fn fetch_all_cidr_bans(
        db: &DbConn,
    ) -> Result<Vec<(cidr_ban::Model, Option<user::Model>)>, DbErr> {
        CidrBan::find()
            .order_by_asc(cidr_ban::Column::RangeBegin)
            .find_also_related(User)
            .all(db)
            .await
    }

    // Get all URL filters in the database.
    // TODO: pagination?
    pub async fn fetch_all_url_filters(
        db: &DbConn,
    ) -> Result<Vec<(url_filter::Model, Option<user::Model>)>, DbErr> {
        UrlFilter::find()
            .order_by_desc(url_filter::Column::Filter)
            .find_also_related(User)
            .all(db)
            .await
    }

    // Check if an IP is banned or not.
    pub async fn check_ip_ban(db: &DbConn, addr: IpAddr) -> Result<bool, DbErr> {
        let octets: [u8; 16] = match addr {
            IpAddr::V4(i) => i.to_ipv6_mapped(),
            IpAddr::V6(i) => i,
        }
        .octets();

        let count = CidrBan::find()
            .filter(
                cidr_ban::Column::RangeBegin
                    .lte(octets.to_vec())
                    .and(cidr_ban::Column::RangeEnd.gte(octets.to_vec())),
            )
            .count(db)
            .await?;

        Ok(count > 0)
    }
}
