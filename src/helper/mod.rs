use crate::AppError;
use actix_web::HttpResponse;
use ethers::{
    abi::{decode as abi_decode, ParamType},
    types::{Address, Bytes},
    utils::hex::decode as hex_decode,
};
use std::env;

const ACCOUNT: &str = "ACCOUNT";
const OTHER_ACCOUNTS: &str = "OTHER_ACCOUNTS";

pub fn get_env_var(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|e| AppError::KeyNotSetError(key.to_owned(), e))
}

pub fn get_all_account_addresses() -> Result<Vec<Address>, AppError> {
    let current = get_env_var(ACCOUNT)?;
    let other_addresses = get_env_var(OTHER_ACCOUNTS)?
        .split_whitespace()
        .map(|item| item.trim().to_lowercase())
        .collect::<Vec<String>>();

    let mut all_parsed = vec![Address::zero(); other_addresses.len() + 1];
    all_parsed[0] = parse_address(&current)?;
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

pub fn parse_address(addr: &str) -> Result<Address, AppError> {
    addr.parse::<Address>()
        .map_err(|e| AppError::AddressParseError(e.into()))
}

pub fn trigger_reload() -> HttpResponse {
    HttpResponse::NoContent()
        .append_header((
            "HX-Trigger",
            "loadResult, loadLastBlockDetails, loadAccountBalances",
        ))
        .finish()
}

pub fn render_error<T: std::error::Error>(e: T) -> HttpResponse {
    let cause = e.source();
    ui_alert(cause.unwrap_or(&e).to_string().as_str())
}

pub fn decode_revert_error(e: &Bytes) -> String {
    let decoded = hex_decode(&e.to_string()[10..])
        .map_err(|e| e.to_string())
        .unwrap();
    let res = abi_decode(&[ParamType::String], decoded.as_slice())
        .map_err(|e| e.to_string())
        .unwrap();
    format!("transaction reverted: {}", res[0])
}
