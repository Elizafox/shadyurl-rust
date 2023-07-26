/* SPDX-License-Identifier: CC0-1.0
 *
 * src/controllers/err.rs
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
    extract::State,
    http::{header::CONTENT_LENGTH, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    BoxError,
};
use itertools::Itertools;
use tower::timeout::error::Elapsed;

use crate::{state::AppState, templates::ErrorTemplate};

// This transforms errors without a body into errors that have one.
// This actually runs as a service, but shrug
pub(crate) async fn transform_error<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let response = next.run(request).await;

    let status_num = response.status().as_u16();
    if status_num < 400 {
        // We only care about client or server errors
        return response;
    }

    let (parts, body) = response.into_parts();

    // Sniff the content-length and see if we have any data
    if let Some(content_length) = parts.headers.get(CONTENT_LENGTH) {
        if let Ok(len) = std::str::from_utf8(content_length.as_bytes())
            .unwrap_or("0")
            .parse::<usize>()
        {
            if len > 0 {
                return Response::from_parts(parts, body);
            }
        }
    }

    let status_string = parts.status.to_string();
    let (error_code, reason) = status_string.splitn(2, ' ').collect_tuple().map_or(
        (format!("{:03}", status_num), status_string.clone()),
        |(x, y)| (x.to_owned(), y.to_owned()),
    );

    let t = ErrorTemplate {
        base_host: &state.base_host,
        error_code: &error_code,
        reason: &reason,
    };
    (parts.status, t).into_response()
}

pub(crate) async fn handle_timeout_error(err: BoxError) -> impl IntoResponse {
    // Let transform_error handle it
    let error_code = if err.is::<Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    error_code.into_response()
}
