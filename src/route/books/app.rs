use axum::{routing::get, Router};

use crate::state::ApiState;

pub fn app() -> Router<ApiState> {
    Router::<ApiState>::new()
        .route("/get_book", get(super::get_book::get_book))
        .route(
            "/get_book_not_found",
            get(super::get_book::get_book_not_found),
        )
        .route(
            "/get_book_id_too_big",
            get(super::get_book::get_book_id_too_big),
        )
}
