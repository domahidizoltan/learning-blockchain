use crate::AppError;
use actix_web::HttpResponse;
use ethers::types::Address;
use std::env;

const ACCOUNT: &str = "ACCOUNT";
const OTHER_ACCOUNTS: &str = "OTHER_ACCOUNTS";

pub fn get_env_var(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|e| AppError::KeyNotSetError(key.to_string(), e))
}

pub fn get_all_account_addresses() -> Result<Vec<Address>, AppError> {
    let current = get_env_var(ACCOUNT)?;
    let other_addresses = get_env_var(OTHER_ACCOUNTS)?
        .split_whitespace()
        .map(|item| item.trim().to_lowercase())
        .collect::<Vec<String>>();

    let mut all_parsed = vec![Address::zero(); other_addresses.len() + 1];
    all_parsed[0] = match current.parse::<Address>() {
        Ok(addr) => addr,
        Err(e) => return Err(AppError::AddressParseError(e.into())),
    };
    for (i, addr) in other_addresses.iter().enumerate() {
        all_parsed[i + 1] = match addr.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => return Err(AppError::AddressParseError(e.into())),
        };
    }

    Ok(all_parsed)
}

pub fn ui_alert(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!(
        "<span class=\"alert alert-error\">⚠ {}</span>",
        msg
    ))
}
