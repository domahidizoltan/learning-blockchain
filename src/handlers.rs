
use actix_web::{get, web, HttpResponse, Responder, HttpRequest, Error};
use tera::Context;
use crate::app::model::State as AppState;

pub fn setup_handlers(cfg: &mut web::ServiceConfig) {
    cfg
        .service(index)
        .service(deploy)
        .service(lab_handler)
        .service(web::resource("/ws/debug").route(web::get().to(debug_events)));
}

#[get("/")]
async fn index(app_state: web::Data<AppState>) -> impl Responder {
    let context = Context::new();
    let rendered = match app_state.tmpl.render("index.html", &context) {
        Ok(rendered) => rendered,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().body(rendered)
}

#[get("/deploy/{name}")]
async fn deploy(name: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("contract_name", name.as_str());
    let rendered = match app_state.tmpl.render("deploying.html", &context) {
        Ok(rendered) => rendered,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(rendered)
}

#[get("/lab/{path}/form")]
async fn lab_handler(path: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let context = Context::new();
    let p = path.replace("-", "_");
    let file_name = format!("lab/{p}/form.html");
    let rendered = match app_state.tmpl.render(&file_name, &context) {
        Ok(rendered) => rendered,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(rendered)
}

async fn debug_events(req: HttpRequest, stream: web::Payload, app_state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let (res, session, _msg_stream) = actix_ws::handle(&req, stream)?;
    app_state.debug_service.set_debug_session(session.clone());
    Ok(res)
}