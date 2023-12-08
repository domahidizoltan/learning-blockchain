use crate::AppError::NoBlockFoundError;
use crate::{client::ethereumclient::EthClient, AppError};
use actix_web::{http::header::HeaderMap, HttpResponse};
use ethers::{
    abi::{decode as abi_decode, ParamType},
    types::{Address, Block, BlockId, Bytes, Transaction, H256},
    utils::{self, hex::decode as hex_decode},
};
use ethers_providers::Middleware;
use std::env;

const ACCOUNT: &str = "ACCOUNT";
const OTHER_ACCOUNTS: &str = "OTHER_ACCOUNTS";
const BLOCK_ID_HEADER: &str = "Blockid";

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
            "loadResult, loadBlockDetails, loadAccountBalances",
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

pub async fn to_block_id(eth: EthClient, input: Option<&str>) -> Result<BlockId, String> {
    match input {
        Some(input) => {
            if input.len() == 66 && input[0..2].starts_with("0x") {
                let hex_decoded = match utils::hex::decode(&input[2..]) {
                    Ok(decoded) => decoded,
                    Err(e) => return Err(e.to_string()),
                };
                let hash = H256::from_slice(&hex_decoded);
                Ok(BlockId::from(hash))
            } else {
                let nr = input.parse::<u64>().unwrap();
                Ok(BlockId::from(nr))
            }
        }
        None => {
            let nr = match eth.get_block_number().await {
                Ok(block_number) => block_number.as_u64(),
                Err(e) => return Err(e.to_string()),
            };
            Ok(BlockId::from(nr))
        }
    }
}

pub async fn get_block(eth: EthClient, input: Option<&str>) -> Result<Block<Transaction>, String> {
    let block_id = match to_block_id(eth.clone(), input).await {
        Ok(block_id) => block_id,
        Err(e) => return Err(e),
    };

    let block = match eth.get_block_with_txs(block_id).await {
        Ok(block) => block,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(block) = block {
        Ok(block)
    } else {
        Err(NoBlockFoundError(input.unwrap_or_default().to_string()).to_string())
    }
}

pub fn get_block_id_from_header_value(headers: &HeaderMap) -> Option<&str> {
    match headers.get(BLOCK_ID_HEADER) {
        Some(block_id) => match block_id.to_str() {
            Ok(block_id) if block_id.is_empty() => Some(block_id),
            _ => None,
        },
        None => None,
    }
}
