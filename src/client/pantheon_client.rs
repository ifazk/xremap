use crate::client::{Client, WindowInfo};
use anyhow::bail;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use zbus::{zvariant, Connection};

pub struct PantheonClient {
    connection: Option<Connection>,
}

impl PantheonClient {
    pub fn new() -> Self {
        Self { connection: None }
    }

    fn connect(&mut self) {
        match block_on(Connection::session()) {
            Ok(connection) => self.connection = Some(connection),
            Err(e) => println!("PantheonClient#connect() failed: {}", e),
        }
    }

    fn get_gala_windows(&mut self) -> Option<Vec<(u64, GalaWindowProperties)>> {
        // self.connect() already called if we got this far
        let connection = match &mut self.connection {
            Some(connection) => connection,
            None => return None,
        };
        if let Ok(message) = block_on(connection.call_method(
            Some("org.pantheon.gala"),
            "/org/pantheon/gala/DesktopInterface",
            Some("org.pantheon.gala.DesktopIntegration"),
            "GetWindows",
            &(),
        )) {
            if let Ok(gala_windows) = message.body().deserialize::<Vec<(u64, GalaWindowProperties)>>() {
                return Some(gala_windows);
            }
        }
        None
    }
}

impl Client for PantheonClient {
    fn supported(&mut self) -> bool {
        self.connect();
        self.current_application().is_some()
    }

    fn current_window(&mut self) -> Option<String> {
        let gala_windows = self.get_gala_windows()?;
        for (_id, window) in gala_windows {
            if window.has_focus {
                return Some(window.title);
            }
        }
        None
    }

    fn current_application(&mut self) -> Option<String> {
        let gala_windows = self.get_gala_windows()?;
        for (_id, window) in gala_windows {
            if window.has_focus {
                return window.sandboxed_app_id.or(Some(window.app_id));
            }
        }
        None
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for PANTHEON")
    }
}

#[derive(Serialize, Deserialize, zvariant::Type)]
#[zvariant(signature = "dict")]
struct GalaWindowProperties {
    #[serde(default)]
    wm_class: String,
    #[serde(default)]
    title: String,
    #[serde(rename = "app-id")]
    app_id: String,
    #[serde(default, rename = "sandboxed-app-id")]
    sandboxed_app_id: Option<String>,
    #[serde(rename = "client-type")]
    client_type: u32,
    #[serde(rename = "is-hidden")]
    is_hidden: bool,
    #[serde(rename = "has-focus")]
    has_focus: bool,
    #[serde(rename = "workspace-index")]
    workspace_index: i32,
    width: u32,
    height: u32,
}
