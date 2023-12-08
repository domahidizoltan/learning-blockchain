use crate::{
    app::model::State as AppState,
    helper,
    lab::{deploy, load_template},
};
use actix_web::{
    web::{self},
    HttpRequest, HttpResponse, Responder,
};
use ethers::contract::abigen;
use serde::Deserialize;
use tera::Context;

#[derive(Deserialize, Debug)]
struct FormData {
    message: String,
}

abigen!(
    TheBlockchainMessenger,
    r#"[
        function updateTheMessage(string)
        function changeCounter()(uint)
        function theMessage()(string)
    ]"#
);

const CONTRACT_NAME: &str = "TheBlockchainMessenger";
const LAB_PATH: &str = "lab/the_blockchain_messenger";
const LAB_BASEURL: &str = "/lab/the-blockchain-messenger";
const CONTRACT_ADDRESS_ENVVAR: &str = "CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER";

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

async fn tx_result_handler(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let result_path = format!("{}/result.html", LAB_PATH);
    let eth = app_state.eth_client.get_client();
    let block_id = helper::get_block_id_from_header_value(req.headers());
    let block_id = match helper::to_block_id(eth, block_id).await {
        Ok(block_id) => block_id,
        Err(e) => return helper::ui_alert(&e),
    };

    let lock = app_state.contracts.lock().await;
    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };
    let contract_address = format!("{:#x}", contract.address());

    let mut context = Context::new();
    context.insert("contract_address", &contract_address);

    let contract = TheBlockchainMessenger::new(contract.address(), contract.client());
    let counter = match contract.change_counter().block(block_id).call().await {
        Ok(counter) => counter,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    let msg = match contract.the_message().block(block_id).call().await {
        Ok(msg) => msg,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("message", &msg);
    context.insert("counter", &counter.as_u64());
    let rendered = match app_state.tmpl.render(&result_path, &context) {
        Ok(rendered) => rendered,
        Err(e) => return helper::render_error(e),
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

    let lock = app_state.contracts.lock().await;
    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };

    let contract = TheBlockchainMessenger::new(contract.address(), contract.client());
    let call = contract.update_the_message(form.message.clone());
    let pending_tx = match call.send().await {
        Ok(receipt) => receipt,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    match pending_tx.await {
        Ok(receipt) => {
            app_state
                .debug_service
                .send_debug_event(&format!("<b>[{CONTRACT_NAME}]</b> receipt: {receipt:?}"))
                .await;
            helper::trigger_reload()
        }
        Err(e) => helper::ui_alert(&e.to_string()),
    }
}
