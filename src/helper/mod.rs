use std::env;
use actix_web::HttpResponse;

pub fn get_env_var(key: &str) -> Result<String, String> {
    match env::var(key) {
        Ok(val) => Ok(val),
        Err(_) => Err(format!("{} must be set", key)),
    }
}

pub fn ui_alert(msg: String) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!("<span class=\"alert alert-error\">⚠ {}</span>", msg))
}