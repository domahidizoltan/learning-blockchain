use crate::{
    app::model::State as AppState,
    helper,
    lab::{deploy, load_template},
};
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use ethers::{
    abi::{decode as abi_decode, ParamType},
    contract::abigen,
    prelude::SignerMiddleware,
    prelude::Wallet,
    types::{Bytes, TransactionReceipt, TransactionRequest, H160, U256},
    utils::hex::decode as hex_decode,
};
use ethers_contract::ContractError;
use ethers_providers::{Http, Middleware};
use k256::Secp256k1;
use serde::Deserialize;
use tera::Context;

type SharedWalletType = SharedWallet<
    SignerMiddleware<ethers_providers::Provider<Http>, Wallet<ecdsa::SigningKey<Secp256k1>>>,
>;

#[derive(Deserialize, Debug)]
enum Action {
    FundContract,
    SetAllowance,
    DenySending,
    TransferToAddress,
}

#[derive(Deserialize, Debug)]
struct FormData {
    action: Action,
    amount: Option<u64>,
    address: Option<String>,
    message: Option<String>,
}

abigen!(
    SharedWallet,
    r#"[
        function proposeNewOwner(address)
        function setAllowance(address, uint)
        function denySending(address)
        function transfer(address, uint, bytes)(bytes) 

        function getContractBalance()(uint)
        function owner()(address)
        function getAllowanceMapAsString()(string)
        function getIsAllowedToSendMapAsString()(string)
        function getGuardianMapAsString()(string)
        function nextOwner()(address)
        function guardiansResetCount()(uint)
    ]"#
);

const CONTRACT_NAME: &str = "SharedWallet";
const LAB_PATH: &str = "lab/shared_wallet";
const LAB_BASEURL: &str = "/lab/shared-wallet";
const CONTRACT_ADDRESS_ENVVAR: &str = "CONTRACT_ADDRESS_SHAREDWALLET";
const CONTRACT_REVERT_ERROR_STRING_SIG: &str = "0x08c379a0";

pub fn setup_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource(LAB_BASEURL).route(web::get().to(load_template_handler)))
        .service(
            web::resource(format!("{}/result", LAB_BASEURL))
                .route(web::get().to(tx_result_handler)),
        )
        .service(
            web::resource(format!("{}/deploy", LAB_BASEURL)).route(web::post().to(deploy_handler)),
        )
        .service(
            web::resource(format!("{}/form", LAB_BASEURL)).route(web::post().to(submit_handler)),
        );
}

async fn load_template_handler(app_state: web::Data<AppState>) -> impl Responder {
    load_template(app_state, LAB_PATH, CONTRACT_NAME).await
}

async fn tx_result_handler(app_state: web::Data<AppState>) -> impl Responder {
    let result_path = format!("{}/result.html", LAB_PATH);

    let lock = match app_state.contracts.read() {
        Ok(lock) => lock,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };
    let contract_address = format!("{:#x}", contract.address());

    let mut context = Context::new();
    context.insert("contract_address", &contract_address);

    let contract = SharedWallet::new(contract.address(), contract.client());

    let client = app_state.eth_client.get_client();
    let contract_balance = match &client.get_balance(contract.address(), None).await {
        Ok(balance) => balance.to_string(),
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("contract_balance", &contract_balance);

    let owner = match contract.owner().call().await {
        Ok(owner) => format!("{:#x}", owner),
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("owner", &owner);

    let allowance = match contract.get_allowance_map_as_string().call().await {
        Ok(allowance) => allowance,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("allowance", &allowance);

    let is_allowed_to_send = match contract.get_is_allowed_to_send_map_as_string().call().await {
        Ok(allowed) => allowed,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("is_allowed_to_send", &is_allowed_to_send);

    let guardian = match contract.get_guardian_map_as_string().call().await {
        Ok(guardian) => guardian,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("guardian", &guardian);

    let next_owner = match contract.next_owner().call().await {
        Ok(owner) => format!("{:#x}", owner),
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("next_owner", &next_owner);

    let guardians_reset_count = match contract.guardians_reset_count().call().await {
        Ok(count) => count,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("guardians_reset_count", &guardians_reset_count.to_string());

    let rendered = match app_state.tmpl.render(&result_path, &context) {
        Ok(rendered) => rendered,
        Err(e) => {
            println!("error rendering template: {:?}", e);
            return helper::ui_alert(&e.to_string());
        }
    };

    HttpResponse::Ok().body(rendered)
}

async fn deploy_handler(app_state: web::Data<AppState>) -> impl Responder {
    deploy(
        app_state,
        CONTRACT_NAME,
        CONTRACT_ADDRESS_ENVVAR,
        LAB_BASEURL,
        (),
    )
    .await
}

async fn submit_handler(
    form: web::Form<FormData>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    app_state
        .debug_service
        .send_debug_event(&format!(
            "<b>[{CONTRACT_NAME}]</b> transaction requested: {form:?}"
        ))
        .await;

    let lock = match app_state.contracts.read() {
        Ok(lock) => lock,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };

    let contract = SharedWallet::new(contract.address(), contract.client());
    let adr = if form.address.is_none() {
        H160::zero()
    } else {
        match helper::parse_address(form.address.clone().unwrap().as_str()) {
            Ok(adr) => adr,
            Err(e) => return helper::ui_alert(&e.to_string()),
        }
    };
    let amount = form.amount.clone().unwrap_or(0);
    let message = form.message.clone().unwrap_or("".to_string());

    let tx_receipt: Result<Option<TransactionReceipt>, String> = match form.action {
        Action::FundContract => fund_contract(contract.address(), amount, &app_state).await,
        Action::SetAllowance => set_allowance(adr, amount, contract).await,
        Action::DenySending => deny_sending(adr, contract).await,
        Action::TransferToAddress => {
            transfer_to_address(adr, amount, message.as_str(), contract).await
        }
    };

    let receipt = match tx_receipt {
        Ok(receipt) => receipt,
        Err(e) => return helper::ui_alert(e.as_str()),
    };

    match receipt {
        Some(receipt) => {
            app_state
                .debug_service
                .send_debug_event(&format!("<b>[{CONTRACT_NAME}]</b> receipt: {receipt:?}"))
                .await;
            trigger_reload()
        }
        None => helper::ui_alert("No receipt for transaction"),
    }
}

fn trigger_reload() -> HttpResponse {
    HttpResponse::NoContent()
        .append_header((
            "HX-Trigger",
            "loadResult, loadLastBlockDetails, loadAccountBalances",
        ))
        .finish()
}

async fn fund_contract(
    contract_address: H160,
    amount: u64,
    app_state: &web::Data<AppState>,
) -> Result<Option<TransactionReceipt>, String> {
    let tx_req = TransactionRequest::new()
        .to(contract_address)
        .value(U256::from(amount));
    let client = app_state.eth_client.get_client();
    let pending_tx_res = client.send_transaction(tx_req, None).await;

    match pending_tx_res {
        Ok(tx) => match tx.await {
            Ok(receipt) => Ok(receipt),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

async fn set_allowance(
    address: H160,
    amount: u64,
    contract: SharedWalletType,
) -> Result<Option<TransactionReceipt>, String> {
    match contract
        .set_allowance(address, U256::from(amount))
        .send()
        .await
    {
        Ok(receipt) => match receipt.await {
            Ok(receipt) => Ok(receipt),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

async fn deny_sending(
    address: H160,
    contract: SharedWalletType,
) -> Result<Option<TransactionReceipt>, String> {
    match contract.deny_sending(address).send().await {
        Ok(receipt) => match receipt.await {
            Ok(receipt) => Ok(receipt),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

async fn transfer_to_address(
    address: H160,
    amount: u64,
    _message: &str,
    contract: SharedWalletType,
) -> Result<Option<TransactionReceipt>, String> {
    match contract
        .transfer(
            address,
            U256::from(amount),
            Bytes::from_static("test".as_bytes()),
        )
        .send()
        .await
    {
        Ok(receipt) => match receipt.await {
            Ok(receipt) => Ok(receipt),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => match e {
            ContractError::Revert(e) => {
                let err = e.to_string();
                match &err[..10] {
                    CONTRACT_REVERT_ERROR_STRING_SIG => {
                        let decoded = hex_decode(&err[10..]).map_err(|e| e.to_string())?;
                        let res = abi_decode(&[ParamType::String], &decoded.as_slice())
                            .map_err(|e| e.to_string())?;
                        Err(format!("transaction reverted: {}", res[0]))
                    }
                    _ => Err(format!("unknown transaction revert error: {}", e)),
                }
            }
            _ => Err(e.to_string()),
        },
    }
}
