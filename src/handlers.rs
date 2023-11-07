use crate::{app::model::State as AppState, helper};
use actix_web::{get, web, Error, HttpRequest, HttpResponse, Responder};
use ethers::types::{Address, H256, U256};
use ethers_providers::Middleware;
use tera::Context;

pub fn setup_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(index)
        .service(deploy_handler)
        .service(lab_handler)
        .service(last_block_details_handler)
        .service(account_balances_handler)
        .service(web::resource("/ws/debug").route(web::get().to(debug_events)));
}

#[get("/")]
async fn index(app_state: web::Data<AppState>) -> impl Responder {
    match app_state.tmpl.render("index.html", &Context::new()) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => {
            println!("error rendering template: {:?}", e);
            helper::ui_alert(&e.to_string())
        }
    }
}

#[get("/deploy/{name}")]
async fn deploy_handler(name: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("contract_name", name.as_str());
    match app_state.tmpl.render("deploying.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => {
            println!("error rendering template: {:?}", e);
            helper::ui_alert(&e.to_string())
        }
    }
}

#[get("/lab/{path}/form")]
async fn lab_handler(path: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("current_address", &app_state.accounts[0]);
    context.insert("other_account_addresses", &app_state.accounts[1..]);

    let p = path.replace('-', "_");
    let file_name = format!("lab/{p}/form.html");
    match app_state.tmpl.render(&file_name, &context) {
        Ok(rendered) => HttpResponse::Ok()
            .append_header(("HX-Trigger", "loadResult"))
            .body(rendered),
        Err(e) => {
            println!("error rendering template: {:?}", e);
            helper::ui_alert(&e.to_string())
        }
    }
}

#[get("/last-block-details")]
async fn last_block_details_handler(app_state: web::Data<AppState>) -> impl Responder {
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
    context.insert("block_number", &block_number);
    context.insert(
        "block_hash",
        &format!("{:#x}", block.hash.unwrap_or_default()),
    );
    context.insert("parent_hash", &format!("{:#x}", block.parent_hash));
    context.insert("block_time", &block.time().unwrap_or_default().to_string());
    context.insert("transaction", &format!("{:#x}", U256::from(tx.as_bytes())));
    context.insert("gas_used", &block.gas_used.as_u64());

    match app_state.tmpl.render("last_block_details.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => {
            println!("error rendering template: {:?}", e);
            helper::ui_alert(&e.to_string())
        }
    }
}

#[get("/account-balances")]
async fn account_balances_handler(app_state: web::Data<AppState>) -> impl Responder {
    let mut balances = vec![(Address::zero(), String::from("0")); app_state.accounts.len()];
    for (i, adr) in app_state.accounts.iter().enumerate() {
        let balance = match app_state
            .eth_client
            .get_client()
            .get_balance(*adr, None)
            .await
        {
            Ok(balance) => balance.to_string(),
            Err(e) => return helper::ui_alert(&e.to_string()),
        };
        balances[i] = (*adr, balance);
    }

    let mut context = Context::new();
    context.insert("account_balances", &balances);

    match app_state.tmpl.render("account_balances.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => {
            println!("error rendering template: {:?}", e);
            helper::ui_alert(&e.to_string())
        }
    }
}

async fn debug_events(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, session, _msg_stream) = actix_ws::handle(&req, stream)?;
    app_state.debug_service.set_debug_session(session);
    Ok(res)
}
