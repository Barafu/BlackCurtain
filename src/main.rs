use std::sync::OnceLock;
use std::time::Duration;

use directories::ProjectDirs;
use eframe::egui::{self, Color32};

mod settings;

static CONFIG_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

fn config_dir() -> &'static std::path::PathBuf {
    CONFIG_DIR.get_or_init(|| {
        ProjectDirs::from("dev", "Barafu Albino", "Black Curtain")
            .expect("could not determine config directory")
            .config_dir()
            .to_path_buf()
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionType {
    Wayland,
    X11,
    Unknown,
}

fn detect_session_type() -> SessionType {
    match std::env::var("XDG_SESSION_TYPE").ok().as_deref() {
        Some("wayland") => SessionType::Wayland,
        Some("x11") => SessionType::X11,
        _ => SessionType::Unknown,
    }
}

/// Notifies the user about an error that prevents using the application.
///
/// Attempts to send a desktop notification via `notify-send` and always
/// prints the message to stderr. It is intentionally infallible: any failure
/// while notifying is silently ignored so that it never produces an error itself.
fn notify_error(message: &str) {
    eprintln!("{message}");
    let _ = std::process::Command::new("notify-send")
        .arg("-u")
        .arg("critical")
        .arg("Black Curtain")
        .arg(message)
        .status();
}

fn main() -> eframe::Result {
    let settings = match settings::Settings::load() {
        Ok(settings) => settings,
        Err(err) => {
            notify_error(&format!("Failed to load settings: {err}"));
            std::process::exit(1);
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([settings.width as f32, settings.height as f32])
            .with_title("Black Curtain")
            .with_app_id("black_curtain"),
        ..Default::default()
    };

    let app = BlackCurtain::new(settings);

    eframe::run_native(
        "Black Curtain",
        options,
        Box::new(move |_cc| Ok(Box::new(app))),
    )
}

/// Application state.
struct BlackCurtain {
    fullscreen: bool,             // true when the window takes the full screen
    always_on_top: bool,          // true when the window stays above others (X11 only)
    show_help: bool,              // true when the help overlay is visible
    hex_input: String,            // user-typed hex colour string (e.g. "#ff8800")
    session_type: SessionType, // detected once at startup, never changes for the process lifetime
    settings: settings::Settings, // persisted window settings
}

impl BlackCurtain {
    fn new(settings: settings::Settings) -> Self {
        Self {
            fullscreen: false,
            always_on_top: false,
            show_help: false,
            hex_input: String::new(),
            session_type: detect_session_type(),
            settings,
        }
    }

    // Help window: a list of controls, color picker, and close button
    fn show_help_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Help")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // --- Controls table ---
                egui::Grid::new("help_grid")
                    .striped(true)
                    .min_col_width(120.0)
                    .show(ui, |ui| {
                        ui.strong("Action");
                        ui.strong("Mouse");
                        ui.strong("Keyboard");
                        ui.end_row();

                        ui.label("Toggle fullscreen");
                        ui.label("Double-click");
                        ui.label("Space");
                        ui.end_row();

                        ui.label("Minimize window");
                        ui.label("Right-click");
                        ui.label("Enter");
                        ui.end_row();

                        ui.label("Show help");
                        ui.label("Middle-click");
                        ui.label("F1");
                        ui.end_row();

                        if self.session_type != SessionType::Wayland {
                            ui.label("Always on top");
                            ui.label("—");
                            ui.label("L");
                            ui.end_row();
                        }
                    });

                // --- Color input row ---
                ui.separator();
                ui.horizontal(|ui| ui.label("Curtain color:"));
                ui.horizontal(|ui| {
                    ui.label("Input here:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.hex_input)
                            .char_limit(7)
                            .desired_width(80.0),
                    );
                    let valid = Color32::from_hex(&self.hex_input).is_ok();
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && valid {
                        if let Ok(c) = Color32::from_hex(&self.hex_input) {
                            self.settings.color = c.to_hex();
                        }
                    }
                    if ui.add_enabled(valid, egui::Button::new("Apply")).clicked() {
                        if let Ok(c) = Color32::from_hex(&self.hex_input) {
                            self.settings.color = c.to_hex();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("or pick here:");
                    let current = Color32::from_hex(&self.settings.color).unwrap_or(Color32::BLACK);
                    let mut srgb = [current.r(), current.g(), current.b()];
                    if ui.color_edit_button_srgb(&mut srgb).changed() {
                        self.settings.color = Color32::from_rgb(srgb[0], srgb[1], srgb[2]).to_hex();
                        self.hex_input = Color32::from_rgb(srgb[0], srgb[1], srgb[2]).to_hex();
                    }
                });
                ui.separator();

                // --- Close button ---
                ui.allocate_space(egui::vec2(0.0, 8.0));
                ui.vertical_centered(|ui| {
                    if ui
                        .add_sized(egui::vec2(120.0, 32.0), egui::Button::new("Close"))
                        .clicked()
                    {
                        if let Err(err) = self.settings.save() {
                            notify_error(&format!("Failed to save settings: {err}"));
                        } else {
                            self.show_help = false;
                        }
                    }
                });
            });
    }
}

impl eframe::App for BlackCurtain {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let color = Color32::from_hex(&self.settings.color).unwrap_or(Color32::BLACK);
        ui.ctx().global_style_mut(move |style| {
            style.visuals.panel_fill = color;
        });

        let cursor_icon = if self.show_help || ui.ctx().input(|i| i.pointer.is_moving()) {
            ui.ctx().request_repaint_after(Duration::from_secs_f32(0.5));
            egui::CursorIcon::Default
        } else {
            egui::CursorIcon::None
        };
        ui.ctx().set_cursor_icon(cursor_icon);

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let response = ui.allocate_rect(ui.max_rect(), egui::Sense::click());

            // Doesn't work because of https://github.com/emilk/egui/issues/7959
            // let left_down = ui.ctx().input(|i| i.pointer.button_down(egui::PointerButton::Primary));
            // if left_down {
            //     if !self.dragging {
            //         self.dragging = true;
            //         ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            //         println!("Drag start");
            //     }
            //     else {
            //         self.dragging = false;
            //     }
            // }
            // if left_down {
            //     println!("Left down");
            // }
            // if any_up {
            //     println!("Drag stop");
            //     self.dragging = false;
            // }

            if response.double_clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                self.fullscreen = !self.fullscreen;
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
            }
            if response.clicked_by(egui::PointerButton::Secondary)
                || (ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.show_help)
            {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            if self.session_type != SessionType::Wayland
                && ui.input(|i| i.key_pressed(egui::Key::L))
            {
                self.always_on_top = !self.always_on_top;
                let level = if self.always_on_top {
                    egui::viewport::WindowLevel::AlwaysOnTop
                } else {
                    egui::viewport::WindowLevel::Normal
                };
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                let title = if self.always_on_top {
                    "Black Curtain (Always on Top)"
                } else {
                    "Black Curtain"
                };
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Title(title.into()));
            }
            if response.clicked_by(egui::PointerButton::Middle)
                || ui.input(|i| i.key_pressed(egui::Key::F1))
            {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.hex_input = Color32::from_hex(&self.settings.color)
                        .unwrap_or(Color32::BLACK)
                        .to_hex();
                }
            }
        });

        if self.show_help {
            self.show_help_window(ui.ctx());
        }
    }
}
