use chrono::{offset::Local, DateTime};
use std::sync::RwLock;

const DATE_FORMAT: &str = "%d/%m/%Y %T";

pub struct DebugService {
    debug_session: RwLock<Option<actix_ws::Session>>,
}

impl DebugService {
    pub fn new() -> Self {
        Self {
            debug_session: RwLock::new(None),
        }
    }

    pub fn set_debug_session(&self, session: actix_ws::Session) {
        let mut ds = self.debug_session.write().unwrap();
        *ds = Some(session);
    }

    pub async fn send_debug_event(&self, msg: String) {
        match self.debug_session.read() {
            Ok(debug_session) => match &*debug_session {
                Some(session) => {
                    log::debug!("••• {}", msg);
                    let now: DateTime<Local> = std::time::SystemTime::now().into();
                    let msg_format = format!(
                        r#"<div id="debug" hx-swap-oob="afterbegin"><p><b>• {} : </b>{}</p></div>"#,
                        now.format(DATE_FORMAT),
                        msg
                    );
                    match session.clone().text(msg_format).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("failed to send debug event: {}", e);
                        }
                    }
                }
                None => {
                    log::warn!("failed to send debug event: no session");
                }
            },
            Err(e) => {
                log::error!("failed to get debug session: {}", e);
            }
        };
    }
}
