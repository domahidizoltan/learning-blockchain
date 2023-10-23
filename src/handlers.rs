use crate::{app::model::State as AppState, helper};
use actix_web::{get, web, Error, HttpRequest, HttpResponse, Responder};
use tera::Context;

pub fn setup_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(index)
        .service(deploy)
        .service(lab_handler)
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
async fn deploy(name: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
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

async fn debug_events(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, session, _msg_stream) = actix_ws::handle(&req, stream)?;
    app_state.debug_service.set_debug_session(session);
    Ok(res)
}
