use std::collections::HashMap;

use axum::{extract::Query, response::Redirect};
use axum_extra::extract::{CookieJar, cookie::Cookie};

pub async fn set_token(Query(query): Query<HashMap<String, String>>) -> (CookieJar, Redirect) {
    let mut jar = CookieJar::new();

    if let Some(token) = query.get("token") {
        let token_cookie = Cookie::build(("token", token.clone())).permanent();
        jar = jar.add(token_cookie);
    }

    (jar, Redirect::to("/"))
}
