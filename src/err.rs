/* SPDX-License-Identifier: CC0-1.0
 *
 * src/err.rs
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

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::templates::ErrorTemplate;

pub(crate) fn respond_not_authorised() -> Response {
    let t = ErrorTemplate {
        error_code: "403",
        reason: "Not authorised",
    };
    (StatusCode::FORBIDDEN, t).into_response()
}

pub(crate) fn respond_not_found() -> Response {
    let t = ErrorTemplate {
        error_code: "404",
        reason: "File not found",
    };
    (StatusCode::NOT_FOUND, t).into_response()
}

pub(crate) fn respond_internal_server_error() -> Response {
    let t = ErrorTemplate {
        error_code: "500",
        reason: "Internal server error",
    };
    (StatusCode::INTERNAL_SERVER_ERROR, t).into_response()
}
