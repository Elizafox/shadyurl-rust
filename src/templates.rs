/* SPDX-License-Identifier: CC0-1.0
 *
 * src/templates.rs
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

use askama::Template;

use entity::url::Model;

#[derive(Template)]
#[template(path = "index.html")]
pub(crate) struct IndexTemplate<'a> {
    pub(crate) base_host: &'a str,
    pub(crate) sitename: &'a str,
    pub(crate) auth_token: &'a str,
}

#[derive(Template)]
#[template(path = "login_landing.html")]
pub(crate) struct LoginTemplate<'a> {
    pub(crate) base_host: &'a str,
    pub(crate) err_str: &'a str,
    pub(crate) sitename: &'a str,
    pub(crate) auth_token: &'a str,
}

#[derive(Template)]
#[template(path = "post.html")]
pub(crate) struct PostTemplate<'a> {
    pub(crate) base_host: &'a str,
    pub(crate) shady_host: &'a str,
    pub(crate) url: &'a str,
    pub(crate) shady: &'a str,
}

#[derive(Template)]
#[template(path = "post_error.html")]
pub(crate) struct PostErrorTemplate<'a> {
    pub(crate) base_host: &'a str,
    pub(crate) url: &'a str,
    pub(crate) reason: &'a str,
}

#[derive(Template)]
#[template(path = "admin.html")]
pub(crate) struct AdminTemplate<'a> {
    pub(crate) base_host: &'a str,
    pub(crate) sitename: &'a str,
    pub(crate) urls: Vec<Model>,
    pub(crate) auth_token: &'a str,
}

#[derive(Template)]
#[template(path = "error.html")]
pub(crate) struct ErrorTemplate<'a> {
    pub(crate) base_host: &'a str,
    pub(crate) error_code: &'a str,
    pub(crate) reason: &'a str,
}
