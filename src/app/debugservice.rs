use std::sync::RwLock;

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
                    let msg_format = format!(r#"<div id="debug" hx-swap-oob="afterbegin">{}<br/></div>"#, msg);
                    session.clone().text(msg_format).await.unwrap();
                },
                None => {
                    log::warn!("failed to send debug event: no session");
                },
            },
            Err(e) => {
                log::error!("failed to get debug session: {}", e);
            },
        };
        
    }
}
