/* SPDX-License-Identifier: CC0-1.0
 *
 * src/error_response.rs
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

use askama_axum::Template;
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    auth::{AuthError, Backend},
    csrf::VerifyCsrfError,
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Csrf(#[from] csrf::CsrfError),

    #[error(transparent)]
    VerifyCsrf(#[from] VerifyCsrfError),

    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    AxumLogin(#[from] axum_login::Error<Backend>),

    #[error(transparent)]
    Session(#[from] tower_sessions::session::Error),

    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),

    #[error("Could not validate URL {}: {}", .1, .0)]
    UrlValidation(String, String),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::VerifyCsrf(e) => ErrorResponse::bad_request(e.to_string().as_ref()),
            Self::UrlValidation(error_reason, url) => {
                ErrorResponse::url_submission(&error_reason, &url)
            }
            Self::NotFound => ErrorResponse::not_found(),
            Self::Unauthorized => ErrorResponse::unauthorized(),
            _ => ErrorResponse::internal_server_error(self.to_string().as_str()),
        }
    }
}

#[derive(Template)]
#[template(path = "errors/code/400.html")]
struct BadRequestTemplate<'a> {
    error_reason: &'a str,
}

#[derive(Template)]
#[template(path = "errors/code/403.html")]
struct UnauthorizedTemplate;

#[derive(Template)]
#[template(path = "errors/code/404.html")]
struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "errors/code/500.html")]
struct InternalServerErrorTemplate<'a> {
    error_reason: &'a str,
}

#[derive(Template)]
#[template(path = "errors/form/url.html")]
struct UrlSubmissionErrorTemplate<'a> {
    error_reason: &'a str,
    url: &'a str,
}

pub struct ErrorResponse;

impl ErrorResponse {
    pub(crate) fn bad_request(error_reason: &str) -> Response<Body> {
        let t = BadRequestTemplate { error_reason };
        (StatusCode::BAD_REQUEST, t).into_response()
    }

    pub(crate) fn unauthorized() -> Response<Body> {
        (StatusCode::UNAUTHORIZED, UnauthorizedTemplate).into_response()
    }

    pub(crate) fn not_found() -> Response<Body> {
        (StatusCode::NOT_FOUND, NotFoundTemplate).into_response()
    }

    pub(crate) fn internal_server_error(error_reason: &str) -> Response<Body> {
        let t = InternalServerErrorTemplate { error_reason };
        (StatusCode::INTERNAL_SERVER_ERROR, t).into_response()
    }

    pub(crate) fn url_submission<'a>(error_reason: &'a str, url: &'a str) -> Response<Body> {
        let t = UrlSubmissionErrorTemplate { error_reason, url };
        (StatusCode::UNPROCESSABLE_ENTITY, t).into_response()
    }
}
