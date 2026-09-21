use adw::prelude::*;

use crate::error::Result;
use crate::paths;
use crate::ui;

pub fn run() -> Result<()> {
    adw::init().map_err(|err| crate::error::Error::detailed("Could not start GTK.", err.to_string()))?;
    let app = adw::Application::builder()
        .application_id(paths::APP_ID)
        .build();
    app.connect_activate(|app| ui::window::present(app));
    app.run();
    Ok(())
}
