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

use ::entity::{url, url::Entity as Url, user, user::Entity as User};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_user_by_username(
        db: &DbConn,
        username: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
    }

    pub async fn find_user_by_id(db: &DbConn, id: i64) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    pub async fn find_url_by_string(db: &DbConn, url: &str) -> Result<Vec<url::Model>, DbErr> {
        Url::find().filter(url::Column::Url.eq(url)).all(db).await
    }

    pub async fn find_url_by_shady_string(
        db: &DbConn,
        shady: &str,
    ) -> Result<Option<url::Model>, DbErr> {
        Url::find()
            .filter(url::Column::Shady.eq(shady))
            .one(db)
            .await
    }

    pub async fn find_url_by_id(db: &DbConn, id: i64) -> Result<Option<url::Model>, DbErr> {
        Url::find_by_id(id).one(db).await
    }

    pub async fn fetch_all_urls(db: &DbConn) -> Result<Vec<url::Model>, DbErr> {
        Url::find().order_by_asc(url::Column::Id).all(db).await
    }
}
