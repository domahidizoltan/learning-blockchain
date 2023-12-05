use chrono::{offset::Local, DateTime};
use futures::lock::Mutex;
use std::sync::Arc;

const DATE_FORMAT: &str = "%d/%m/%Y %T";

pub struct DebugService {
    debug_session: Arc<Mutex<Option<actix_ws::Session>>>,
}

impl Clone for DebugService {
    fn clone(&self) -> Self {
        Self {
            debug_session: Arc::clone(&self.debug_session),
        }
    }
}

impl Default for DebugService {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugService {
    pub fn new() -> Self {
        Self {
            debug_session: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_debug_session(&self, session: actix_ws::Session) {
        let mut ds = self.debug_session.lock().await;
        *ds = Some(session);
    }

    pub async fn send_debug_event(&self, msg: &str) {
        if let Some(debug_session) = self.debug_session.try_lock() {
            match debug_session.as_ref() {
                Some(session) => {
                    log::debug!("••• {}", msg);
                    let now: DateTime<Local> = std::time::SystemTime::now().into();
                    let msg_format = format!(
                        r#"<div id="debug" hx-swap-oob="afterbegin"><p><b>• {} : </b>{}</p></div>"#,
                        now.format(DATE_FORMAT),
                        msg
                    );
                    if let Some(err) = session.clone().text(msg_format).await.err() {
                        log::error!("failed to send debug event: {msg} error: {err}");
                    }
                }
                None => log::warn!("failed to send debug event (no session): {msg}"),
            }
        }
    }
}
