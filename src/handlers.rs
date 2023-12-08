use crate::{app::model::State as AppState, helper};
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder};
use ethers::types::Address;
use ethers_providers::Middleware;
use tera::Context;

pub fn setup_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(index)
        .service(deploy_handler)
        .service(lab_handler)
        .service(load_block_details_handler)
        .service(block_details_handler)
        .service(account_balances_handler)
        .service(web::resource("/ws/debug").route(web::get().to(debug_events)));
}

#[get("/")]
async fn index(app_state: web::Data<AppState>) -> impl Responder {
    match app_state.tmpl.render("index.html", &Context::new()) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => helper::render_error(e),
    }
}

#[get("/deploy/{name}")]
async fn deploy_handler(name: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("contract_name", name.as_str());
    match app_state.tmpl.render("deploying.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => helper::render_error(e),
    }
}

#[get("/lab/{path:the-blockchain-messenger|smart-money|shared-wallet}/form")]
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
        Err(e) => helper::render_error(e),
    }
}

#[post("/load-block-details")]
async fn load_block_details_handler() -> impl Responder {
    helper::trigger_reload()
}

#[get("/block-details")]
async fn block_details_handler(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let eth = app_state.eth_client.get_client();
    let block_id = helper::get_block_id_from_header_value(req.headers());
    let block = match helper::get_block(eth, block_id).await {
        Ok(block) => block,
        Err(e) => return helper::ui_alert(&e),
    };

    let tx = block.transactions.get(0).unwrap();

    let mut context = Context::new();
    context.insert("block_number", &block.number.unwrap_or_default().as_u64());
    context.insert(
        "block_hash",
        &format!("{:#x}", block.hash.unwrap_or_default()),
    );
    context.insert("parent_hash", &format!("{:#x}", block.parent_hash));
    context.insert("block_time", &block.time().unwrap_or_default().to_string());
    context.insert("transaction", &format!("{:#x}", tx.hash));
    context.insert("gas_used", &block.gas_used.as_u64());

    match app_state.tmpl.render("block_details.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => helper::render_error(e),
    }
}

#[get("/account-balances")]
async fn account_balances_handler(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let eth = app_state.eth_client.get_client();
    let block_id_option = helper::get_block_id_from_header_value(req.headers());
    let block_id = match helper::to_block_id(eth.clone(), block_id_option).await {
        Ok(block_id) => Some(block_id),
        Err(e) => return helper::ui_alert(&e),
    };

    let mut balances = vec![(Address::zero(), String::from("0")); app_state.accounts.len()];
    for (i, adr) in app_state.accounts.iter().enumerate() {
        let balance = match eth.clone().get_balance(*adr, block_id).await {
            Ok(balance) => balance.to_string(),
            Err(e) => return helper::ui_alert(&e.to_string()),
        };
        balances[i] = (*adr, balance);
    }

    let mut context = Context::new();
    context.insert("account_balances", &balances);

    match app_state.tmpl.render("account_balances.html", &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(e) => helper::render_error(e),
    }
}

async fn debug_events(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, session, _msg_stream) = actix_ws::handle(&req, stream)?;
    app_state.debug_service.set_debug_session(session).await;
    Ok(res)
}
