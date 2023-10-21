use crate::{app::model::State as AppState, helper};
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use ethers::{
    contract::abigen,
    types::{H256, U256},
};
use ethers_providers::Middleware;
use serde::Deserialize;
use std::path::Path;
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
    let readme_path = format!("src/{}/README.md", LAB_PATH);
    let template_path = format!("{}/template.html", LAB_PATH);

    let html = match markdown::file_to_html(Path::new(&readme_path)) {
        Ok(html) => html,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    let mut context = Context::new();
    context.insert("contract_name", CONTRACT_NAME);
    context.insert("readme", &html);

    let rendered = match app_state.tmpl.render(&template_path, &context) {
        Ok(rendered) => rendered,
        Err(e) => {
            println!("error rendering template: {:?}", e);
            return helper::ui_alert(&e.to_string());
        }
    };

    HttpResponse::Ok().body(rendered)
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

    let eth = app_state.eth_client.get_client();

    let block_number = match eth.get_block_number().await {
        Ok(block_number) => block_number.as_u64(),
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    let block = match eth.get_block(block_number).await {
        Ok(block) => block.unwrap_or_default(),
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    let zero = H256::zero();
    let tx = block.transactions.get(0).unwrap_or(&zero);

    let mut context = Context::new();
    context.insert("contract_address", &contract_address);
    context.insert("block_number", &block_number);
    context.insert(
        "block_hash",
        &format!("{:#x}", block.hash.unwrap_or_default()),
    );
    context.insert("parent_hash", &format!("{:#x}", block.parent_hash));
    context.insert("block_time", &block.time().unwrap_or_default().to_string());
    context.insert("transaction", &format!("{:#x}", U256::from(tx.as_bytes())));
    context.insert("gas_used", &block.gas_used.as_u64());

    let messenger = TheBlockchainMessenger::new(contract.address(), contract.client());
    let counter = match messenger.change_counter().call().await {
        Ok(counter) => counter,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    let msg = match messenger.the_message().call().await {
        Ok(msg) => msg,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("message", &msg);
    context.insert("counter", &counter.as_u64());
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
    let mut lock = match app_state.contracts.write() {
        Ok(lock) => lock,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    if let Some(contract) = lock.get(CONTRACT_NAME) {
        println!(
            "contract {} already deployed: {:?}",
            CONTRACT_NAME, contract
        );
    } else {
        let contract = match helper::get_env_var(CONTRACT_ADDRESS_ENVVAR) {
            Ok(adr) => {
                app_state
                    .debug_service
                    .send_debug_event(&format!(
                        "recreating contract {}.sol from address {}",
                        CONTRACT_NAME, adr
                    ))
                    .await;
                match app_state
                    .eth_client
                    .contract_from_address(CONTRACT_NAME, adr.as_str())
                    .await
                {
                    Ok(contract) => contract,
                    Err(e) => return helper::ui_alert(&e.to_string()),
                }
            }
            Err(_) => {
                app_state
                    .debug_service
                    .send_debug_event(&format!("deploying contract {}.sol ...", CONTRACT_NAME))
                    .await;
                match app_state.eth_client.deploy_contract(CONTRACT_NAME).await {
                    Ok(contract) => {
                        app_state
                            .debug_service
                            .send_debug_event(&format!(
                                "{}.sol deployed to address {:#x}",
                                CONTRACT_NAME,
                                contract.address()
                            ))
                            .await;
                        contract
                    }
                    Err(e) => return helper::ui_alert(&e.to_string()),
                }
            }
        };

        lock.insert(CONTRACT_NAME.to_owned(), contract);
    }

    HttpResponse::SeeOther()
        .append_header(("Location", LAB_BASEURL.to_owned() + "/form"))
        .finish()
}

async fn submit_handler(
    form: web::Form<FormData>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    app_state
        .debug_service
        .send_debug_event(&format!("update request received: {:?}", form))
        .await;

    let lock = match app_state.contracts.read() {
        Ok(lock) => lock,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };

    let messenger = TheBlockchainMessenger::new(contract.address(), contract.client());
    let call = messenger.update_the_message(form.message.clone());
    let pending_tx = match call.send().await {
        Ok(receipt) => receipt,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    match pending_tx.await {
        Ok(receipt) => {
            app_state
                .debug_service
                .send_debug_event(&format!("receipt: {:?}", receipt))
                .await;
            HttpResponse::NoContent()
                .append_header(("HX-Trigger", "loadResult"))
                .finish()
        }
        Err(e) => helper::ui_alert(&e.to_string()),
    }
}
