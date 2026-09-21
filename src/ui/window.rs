use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use gtk::gdk;
use gtk::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::gio;
use image::GenericImageView;

use crate::error::Error;
use crate::generate::{self, GenerateOptions};
use crate::paths;
use crate::settings::Settings;
use crate::spec::{self, Target};
use crate::ui::{self, dialogs};

struct State {
    settings: RefCell<Settings>,
    source: RefCell<Option<PathBuf>>,
    master: RefCell<Option<image::DynamicImage>>,
}

#[derive(Clone)]
struct Widgets {
    window: adw::ApplicationWindow,
    toast: adw::ToastOverlay,
    picture: gtk::Picture,
    drop_zone: gtk::Box,
    info: gtk::Label,
    warn: gtk::Label,
    target: gtk::DropDown,
    padding: gtk::Scale,
    background: gtk::DropDown,
    keep_alpha: gtk::Switch,
    output: gtk::Label,
    generate: gtk::Button,
    safe_area: gtk::DrawingArea,
}

pub fn present(app: &adw::Application) {
    ui::load_css();
    ui::register_icons();
    let settings = Settings::load();
    ui::apply_theme(settings.theme);
    if let Some(win) = app.active_window() {
        win.present();
        return;
    }

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(paths::APP_NAME)
        .default_width(1040)
        .default_height(760)
        .build();
    window.set_icon_name(Some(paths::APP_ICON));

    let toast = adw::ToastOverlay::new();
    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    let open = icon_btn("document-open-symbolic", "Open master image (Ctrl+O)");
    let settings_btn = icon_btn("emblem-system-symbolic", "Settings");
    let about = icon_btn("help-about-symbolic", "About");
    header.pack_start(&open);
    header.pack_end(&settings_btn);
    header.pack_end(&about);
    toolbar.add_top_bar(&header);

    let kicker = gtk::Label::new(Some("SHADOW ICON FACTORY"));
    kicker.add_css_class("brand-kicker");
    kicker.set_xalign(0.0);

    let picture = gtk::Picture::new();
    picture.set_can_shrink(true);
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    let safe_area = gtk::DrawingArea::new();
    safe_area.set_hexpand(true);
    safe_area.set_vexpand(true);
    safe_area.set_can_target(false);
    let overlay = gtk::Overlay::new();
    overlay.add_css_class("preview-frame");
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);
    overlay.set_child(Some(&picture));
    overlay.add_overlay(&safe_area);

    let info = gtk::Label::new(Some("Drop a PNG, JPEG, WebP, or SVG master. 1024×1024 recommended."));
    info.set_wrap(true);
    info.set_xalign(0.0);
    info.add_css_class("dim-label");
    let warn = gtk::Label::new(None);
    warn.set_wrap(true);
    warn.set_xalign(0.0);
    warn.add_css_class("warning");

    let drop_zone = gtk::Box::new(gtk::Orientation::Vertical, 8);
    drop_zone.set_hexpand(true);
    drop_zone.set_vexpand(true);
    drop_zone.append(&kicker);
    drop_zone.append(&overlay);
    drop_zone.append(&info);
    drop_zone.append(&warn);

    let side = gtk::Box::new(gtk::Orientation::Vertical, 10);
    side.set_width_request(280);
    side.append(&heading("Target"));
    let target = gtk::DropDown::from_strings(&["iOS", "macOS", "Linux", "Web", "Android", "All"]);
    target.set_selected(settings.last_target.min(5));
    target.set_tooltip_text(Some("Which icon set to generate"));
    side.append(&target);

    side.append(&heading("Padding"));
    let padding = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 0.25, 0.01);
    padding.set_value(settings.padding as f64);
    padding.set_draw_value(true);
    padding.set_tooltip_text(Some("Transparent inset around the master (0–25%)"));
    side.append(&padding);

    side.append(&heading("Background"));
    let background = gtk::DropDown::from_strings(&["Transparent", "White", "Black", "Brand indigo"]);
    background.set_selected(if settings.keep_transparency { 0 } else { 1 });
    background.set_tooltip_text(Some("Fill behind transparent pixels"));
    side.append(&background);

    let keep_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let keep_lab = gtk::Label::new(Some("Keep transparency"));
    keep_lab.set_xalign(0.0);
    keep_lab.set_hexpand(true);
    let keep_alpha = gtk::Switch::new();
    keep_alpha.set_active(settings.keep_transparency);
    keep_alpha.set_valign(gtk::Align::Center);
    keep_alpha.set_tooltip_text(Some("Leave alpha when the background is transparent"));
    keep_row.append(&keep_lab);
    keep_row.append(&keep_alpha);
    side.append(&keep_row);

    side.append(&heading("Output folder"));
    let output = gtk::Label::new(Some(&paths::display_home_path(
        settings
            .last_output
            .as_deref()
            .unwrap_or(&paths::default_output()),
    )));
    output.set_xalign(0.0);
    output.set_wrap(true);
    output.set_selectable(true);
    let pick_out = gtk::Button::with_label("Choose folder…");
    pick_out.set_tooltip_text(Some("Where icon sets are written"));
    side.append(&output);
    side.append(&pick_out);

    let generate = gtk::Button::with_label("Generate icon set");
    generate.add_css_class("suggested-action");
    generate.set_tooltip_text(Some("Write validated icon files (does not overwrite without asking)"));
    generate.set_sensitive(false);
    side.append(&generate);

    let note = gtk::Label::new(Some(
        "Squares only. No fake OS rounded corners. Sizes larger than the master are skipped, never silently upscaled. Existing App Icon sets need confirmation.",
    ));
    note.set_wrap(true);
    note.add_css_class("dim-label");
    note.set_xalign(0.0);
    side.append(&note);

    let pane = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    pane.set_margin_top(12);
    pane.set_margin_bottom(12);
    pane.set_margin_start(16);
    pane.set_margin_end(16);
    pane.append(&drop_zone);
    pane.append(&side);
    toolbar.set_content(Some(&pane));
    toast.set_child(Some(&toolbar));
    window.set_content(Some(&toast));

    let widgets = Rc::new(Widgets {
        window: window.clone(),
        toast,
        picture,
        drop_zone,
        info,
        warn,
        target,
        padding,
        background,
        keep_alpha,
        output,
        generate: generate.clone(),
        safe_area: safe_area.clone(),
    });
    let state = Rc::new(State {
        settings: RefCell::new(settings),
        source: RefCell::new(None),
        master: RefCell::new(None),
    });

    safe_area.set_draw_func(|_, cr, w, h| {
        let inset = (w.min(h) as f64) * 0.10;
        cr.set_source_rgba(0.37, 0.91, 0.96, 0.45);
        cr.set_line_width(2.0);
        cr.rectangle(inset, inset, w as f64 - inset * 2.0, h as f64 - inset * 2.0);
        let _ = cr.stroke();
        cr.set_source_rgba(0.99, 0.90, 0.54, 0.85);
        cr.select_font_face("Sans", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Bold);
        cr.set_font_size(11.0);
        let _ = cr.move_to(inset + 6.0, inset + 16.0);
        let _ = cr.show_text("Safe area");
    });

    let w = widgets.clone();
    let s = state.clone();
    open.connect_clicked(move |_| pick_master(&w, &s));
    let w = widgets.clone();
    let s = state.clone();
    pick_out.connect_clicked(move |_| pick_output(&w, &s));
    let w = widgets.clone();
    let s = state.clone();
    generate.connect_clicked(move |_| start_generate(&w, &s, false));
    let w = widgets.clone();
    let s = state.clone();
    widgets.target.connect_selected_notify(move |_| {
        s.settings.borrow_mut().last_target = w.target.selected();
        refresh_warn(&w, &s);
    });
    let s = state.clone();
    let win = widgets.window.clone();
    settings_btn.connect_clicked(move |_| {
        let cur = s.settings.borrow().clone();
        let s = s.clone();
        dialogs::show_settings(&win, &cur, move |updated| {
            ui::apply_theme(updated.theme);
            s.settings.replace(updated);
        });
    });
    let win = widgets.window.clone();
    about.connect_clicked(move |_| dialogs::show_about(&win));

    setup_drop(&widgets, &state);
    add_shortcuts(&window, &widgets, &state);
    window.present();
}

fn heading(t: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(t));
    l.add_css_class("heading");
    l.set_xalign(0.0);
    l
}

fn icon_btn(name: &str, tip: &str) -> gtk::Button {
    let b = gtk::Button::from_icon_name(name);
    b.set_tooltip_text(Some(tip));
    b
}

fn setup_drop(widgets: &Rc<Widgets>, state: &Rc<State>) {
    let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    let zone = widgets.drop_zone.clone();
    target.connect_enter(move |_, _, _| {
        zone.add_css_class("drop-hover");
        gdk::DragAction::COPY
    });
    let zone = widgets.drop_zone.clone();
    target.connect_leave(move |_| zone.remove_css_class("drop-hover"));
    let widgets_d = widgets.clone();
    let state_d = state.clone();
    target.connect_drop(move |_, value, _, _| {
        widgets_d.drop_zone.remove_css_class("drop-hover");
        if let Ok(list) = value.get::<gdk::FileList>() {
            let files: Vec<PathBuf> = list.files().iter().filter_map(|f| f.path()).collect();
            if files.is_empty() {
                dialogs::show_error(
                    &widgets_d.window,
                    &Error::user("That drop was not a local file."),
                );
                return true;
            }
            if files.iter().any(|p| p.is_dir()) {
                dialogs::show_error(
                    &widgets_d.window,
                    &Error::user("Drop a master image file, not a folder."),
                );
                return true;
            }
            if files.len() > 1 {
                widgets_d
                    .toast
                    .add_toast(adw::Toast::new("Using the first file as the master."));
            }
            if let Some(path) = files.into_iter().next() {
                load_master(&widgets_d, &state_d, path);
            }
            return true;
        }
        false
    });
    widgets.drop_zone.add_controller(target);
}

fn add_shortcuts(window: &adw::ApplicationWindow, widgets: &Rc<Widgets>, state: &Rc<State>) {
    let controller = gtk::ShortcutController::new();
    let w = widgets.clone();
    let s = state.clone();
    let action = gio::SimpleAction::new("open", None);
    action.connect_activate(move |_, _| pick_master(&w, &s));
    window.add_action(&action);
    controller.add_shortcut(gtk::Shortcut::new(
        Some(gtk::ShortcutTrigger::parse_string("<Control>o").unwrap()),
        Some(gtk::NamedAction::new("win.open")),
    ));
    window.add_controller(controller);
}

fn pick_master(widgets: &Rc<Widgets>, state: &Rc<State>) {
    let dialog = gtk::FileDialog::new();
    dialog.set_title("Open master image");
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Images"));
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    filter.add_mime_type("image/webp");
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.png");
    filter.add_pattern("*.jpg");
    filter.add_pattern("*.jpeg");
    filter.add_pattern("*.webp");
    filter.add_pattern("*.svg");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    let widgets = widgets.clone();
    let state = state.clone();
    let win = widgets.window.clone();
    dialog.open(Some(&win), gio::Cancellable::NONE, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                load_master(&widgets, &state, path);
            }
        }
    });
}

fn pick_output(widgets: &Rc<Widgets>, state: &Rc<State>) {
    let dialog = gtk::FileDialog::new();
    dialog.set_title("Choose output folder");
    let widgets = widgets.clone();
    let state = state.clone();
    let win = widgets.window.clone();
    dialog.select_folder(Some(&win), gio::Cancellable::NONE, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                widgets
                    .output
                    .set_text(&paths::display_home_path(&path));
                state.settings.borrow_mut().last_output = Some(path);
                let _ = state.settings.borrow().save();
            }
        }
    });
}

fn load_master(widgets: &Widgets, state: &State, path: PathBuf) {
    match generate::load_master(&path) {
        Ok(img) => {
            let (w, h) = img.dimensions();
            if w == 0 || h == 0 {
                dialogs::show_error(
                    &widgets.window,
                    &Error::user("That image has no pixels."),
                );
                return;
            }
            show_preview(widgets, &img);
            widgets.info.set_text(&format!(
                "{}  ·  {w}×{h}  ·  drop another file to replace",
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string())
            ));
            state.master.replace(Some(img));
            state.source.replace(Some(path));
            widgets.generate.set_sensitive(true);
            refresh_warn(widgets, state);
        }
        Err(err) => dialogs::show_error(&widgets.window, &err),
    }
}

fn show_preview(widgets: &Widgets, img: &image::DynamicImage) {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixbuf = Pixbuf::from_mut_slice(
        rgba.into_raw(),
        Colorspace::Rgb,
        true,
        8,
        w as i32,
        h as i32,
        w as i32 * 4,
    );
    widgets
        .picture
        .set_paintable(Some(&gdk::Texture::for_pixbuf(&pixbuf)));
    widgets.safe_area.queue_draw();
}

fn refresh_warn(widgets: &Widgets, state: &State) {
    let Some(img) = state.master.borrow().as_ref().cloned() else {
        widgets.warn.set_text("");
        return;
    };
    let (w, h) = img.dimensions();
    let min_side = w.min(h);
    let target = Target::from_index(widgets.target.selected());
    let need = spec::max_required(target);
    if min_side < need {
        widgets.warn.set_text(&format!(
            "Master short side is {min_side}px. {need}px is needed for every {} size. Larger slots will be skipped — never silently upscaled.",
            target.label()
        ));
    } else {
        widgets.warn.set_text("");
    }
}

fn start_generate(widgets: &Rc<Widgets>, state: &Rc<State>, overwrite: bool) {
    let Some(master) = state.master.borrow().clone() else {
        dialogs::show_error(&widgets.window, &Error::user("Open a master image first."));
        return;
    };
    let Some(source) = state.source.borrow().clone() else {
        return;
    };
    let output = state
        .settings
        .borrow()
        .last_output
        .clone()
        .unwrap_or_else(paths::default_output);
    let occupied = output.join("ios").exists()
        || output.join("macos").exists()
        || output.join("linux").exists()
        || output.join("web").exists()
        || output.join("android").exists();
    if occupied && !overwrite {
        let w = widgets.clone();
        let s = state.clone();
        dialogs::confirm(
            &widgets.window,
            "Overwrite existing icon set?",
            &format!(
                "{} already has generated folders. Overwrite those icon files?",
                paths::display_home_path(&output)
            ),
            move || start_generate(&w, &s, true),
        );
        return;
    }
    let opts = GenerateOptions {
        target: Target::from_index(widgets.target.selected()),
        padding: widgets.padding.value() as f32,
        background: bg_rgba(widgets.background.selected()),
        keep_transparency: widgets.keep_alpha.is_active(),
        output: output.clone(),
        overwrite_xcassets: overwrite,
    };
    widgets.generate.set_sensitive(false);
    match generate::generate(&master, &source, &opts) {
        Ok(report) => {
            let wrote = report.files.iter().filter(|f| !f.upscale_blocked).count();
            let skipped = report.files.iter().filter(|f| f.upscale_blocked).count();
            let mut msg = format!("Wrote {wrote} files to {}", paths::display_home_path(&report.output_root));
            if skipped > 0 {
                msg.push_str(&format!(" · skipped {skipped} oversized slots"));
            }
            widgets.toast.add_toast(adw::Toast::new(&msg));
            if !report.warnings.is_empty() {
                dialogs::show_error(
                    &widgets.window,
                    &Error::user(report.warnings.join("\n")),
                );
            }
            state.settings.borrow_mut().last_output = Some(output);
            state.settings.borrow_mut().last_target = widgets.target.selected();
            state.settings.borrow_mut().padding = widgets.padding.value() as f32;
            state.settings.borrow_mut().keep_transparency = widgets.keep_alpha.is_active();
            let _ = state.settings.borrow().save();
        }
        Err(err) => dialogs::show_error(&widgets.window, &err),
    }
    widgets.generate.set_sensitive(state.master.borrow().is_some());
}

fn bg_rgba(index: u32) -> [u8; 4] {
    match index {
        1 => [255, 255, 255, 255],
        2 => [0, 0, 0, 255],
        3 => [27, 23, 64, 255],
        _ => [0, 0, 0, 0],
    }
}
