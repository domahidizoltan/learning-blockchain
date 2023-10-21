use crate::AppError;
use actix_web::HttpResponse;
use std::env;

pub fn get_env_var(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|e| AppError::KeyNotSetError(key.to_string(), e))
}

pub fn ui_alert(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!(
        "<span class=\"alert alert-error\">⚠ {}</span>",
        msg
    ))
}
