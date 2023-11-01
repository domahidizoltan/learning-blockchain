use crate::{app::model::State as AppState, helper, lab::{load_template, deploy}};
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use ethers::{
    contract::abigen,
    types::Address,
};
use serde::Deserialize;
use tera::Context;

// #[derive(Deserialize, Debug)]
// enum Action {
//     Deposit,
//     WithdrawAll,
//     WithdrawToAddress,
// }

#[derive(Deserialize, Debug)]
struct FormData {
    // action: Action,
    // amount: u64,
    // to_address: String,
}

abigen!(
    SharedWallet,
    r#"[
        function proposeNewOwner(address)
        function setAllowance(address, uint)
        function denySending(address)
        function transfer(address, uint, bytes)(bytes) 

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
    deploy(app_state, CONTRACT_NAME, CONTRACT_ADDRESS_ENVVAR, LAB_BASEURL).await
}

async fn submit_handler(
    form: web::Form<FormData>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    app_state
        .debug_service
        .send_debug_event(&format!("<b>[{CONTRACT_NAME}]</b> transaction requested: {form:?}"))
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
    // let call = match form.action {
    //     Action::Deposit => contract.deposit().value(form.amount),
    //     Action::WithdrawAll => contract.withdraw_all(),
    //     Action::WithdrawToAddress => {
    //         let adr = match form.to_address.parse::<Address>() {
    //             Ok(adr) => adr,
    //             Err(e) => return helper::ui_alert(&e.to_string()),
    //         };
    //         contract.withdraw_to_address(adr)
    //     }
    // };
    // let pending_tx = match call.send().await {
    //     Ok(receipt) => receipt,
    //     Err(e) => return helper::ui_alert(&e.to_string()),
    // };

    // match pending_tx.await {
    //     Ok(receipt) => {
    //         app_state
    //             .debug_service
    //             .send_debug_event(&format!("<b>[{CONTRACT_NAME}]</b> receipt: {receipt:?}"))
    //             .await;
    //         HttpResponse::NoContent()
    //             .append_header(("HX-Trigger", "loadResult, loadLastBlockDetails, loadAccountBalances"))
    //             .finish()
    //     }
    //     Err(e) => helper::ui_alert(&e.to_string()),
    // }
    return HttpResponse::NoContent()
        .append_header(("HX-Trigger", "loadResult, loadLastBlockDetails, loadAccountBalances"))
        .finish()
}
