pub mod shared_wallet;
pub mod smart_money;
pub mod the_blockchain_messenger;

pub use shared_wallet::main::setup_handlers as shared_wallet_handlers;
pub use smart_money::main::setup_handlers as smart_money_handlers;
pub use the_blockchain_messenger::main::setup_handlers as the_blockchain_messenger_handlers;

use crate::{app::model::State as AppState, helper};
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use std::path::Path;
use tera::Context;

async fn load_template(app_state: web::Data<AppState>, lab_path: &str, contract_name: &str) -> impl Responder {
    let readme_path = format!("src/{}/README.md", lab_path);
    let template_path = format!("{}/template.html", lab_path);

    let html = match markdown::file_to_html(Path::new(&readme_path)) {
        Ok(html) => html,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    let mut context = Context::new();
    context.insert("contract_name", contract_name);
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

async fn deploy(app_state: web::Data<AppState>, contract_name: &str, contract_address_envvar: &str, lab_baseurl: &str) -> impl Responder {
    let mut lock = match app_state.contracts.write() {
        Ok(lock) => lock,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    if let Some(contract) = lock.get(contract_name) {
        println!(
            "contract {} already deployed: {:?}",
            contract_name, contract
        );
    } else {
        let contract = match helper::get_env_var(contract_address_envvar) {
            Ok(adr) => {
                app_state
                    .debug_service
                    .send_debug_event(&format!(
                        "<b>[{contract_name}]</b> recreating contract {contract_name}.sol from address {adr}"))
                    .await;
                match app_state
                    .eth_client
                    .contract_from_address(contract_name, adr.as_str())
                    .await
                {
                    Ok(contract) => contract,
                    Err(e) => return helper::ui_alert(&e.to_string()),
                }
            }
            Err(_) => {
                app_state
                    .debug_service
                    .send_debug_event(&format!("<b>[{contract_name}]</b> deploying contract {contract_name}.sol ..."))
                    .await;
                match app_state.eth_client.deploy_contract(contract_name).await {
                    Ok(contract) => {
                        let adr = contract.address();
                        app_state
                            .debug_service
                            .send_debug_event(&format!(
                                "<b>[{contract_name}]</b> {contract_name}.sol deployed to address {adr:#x}"))
                            .await;
                        contract
                    }
                    Err(e) => return helper::ui_alert(&e.to_string()),
                }
            }
        };

        lock.insert(contract_name.to_owned(), contract);
    }

    HttpResponse::SeeOther()
        .append_header(("Location", lab_baseurl.to_owned() + "/form"))
        .finish()
}
