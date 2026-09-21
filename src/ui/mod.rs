pub mod dialogs;
pub mod window;

use crate::paths;
use crate::settings::Theme;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub fn register_icons() {
    gtk::Window::set_default_icon_name(paths::APP_ICON);
    if let Some(display) = gtk::gdk::Display::default() {
        let theme = gtk::IconTheme::for_display(&display);
        if let Ok(cwd) = std::env::current_dir() {
            theme.add_search_path(cwd.join("data/icons"));
        }
        if let Some(data) = dirs::data_local_dir() {
            theme.add_search_path(data.join("icons"));
        }
    }
}

pub fn apply_theme(theme: Theme) {
    adw::StyleManager::default().set_color_scheme(match theme {
        Theme::System => adw::ColorScheme::Default,
        Theme::Light => adw::ColorScheme::ForceLight,
        Theme::Dark => adw::ColorScheme::ForceDark,
    });
}
