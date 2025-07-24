use axum::response::Redirect;
use axum_extra::extract::{CookieJar, cookie::Cookie};

pub async fn clear_token() -> (CookieJar, Redirect) {
    let mut jar = CookieJar::new();

    let token_cookie = Cookie::build("token").removal();
    jar = jar.add(token_cookie);

    (jar, Redirect::to("/"))
}
