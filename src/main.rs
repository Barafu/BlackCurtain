use std::sync::OnceLock;
use std::time::Duration;

use directories::ProjectDirs;
use eframe::egui::{self, Color32};

mod settings;

use settings::{
    DEFAULT_HEIGHT, DEFAULT_SIZE_LABEL, DEFAULT_WIDTH, MAX_WINDOW_SIZE, MIN_WINDOW_HEIGHT,
    MIN_WINDOW_WIDTH,
};

/// Application version, taken from Cargo.toml at compile time.
const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    fullscreen: bool,                 // true when the window takes the full screen
    always_on_top: bool,              // true when the window stays above others (X11 only)
    show_help: bool,                  // true when the help overlay is visible
    hex_input: String, // user-typed hex digits, never showing the leading '#' (e.g. "ff8800")
    hex_input_color: Option<Color32>, // parsed `hex_input`, kept in sync with it (None = invalid)
    color: Color32,    // parsed `settings.color`, cached to avoid reparsing every frame
    session_type: SessionType, // detected once at startup, never changes for the process lifetime
    settings: settings::Settings, // persisted settings
}

impl BlackCurtain {
    fn new(mut settings: settings::Settings) -> Self {
        let show_help = settings.first_run;
        if show_help {
            settings.first_run = false;
            if let Err(err) = settings.save() {
                notify_error(&format!("Failed to save settings: {err}"));
            }
        }
        Self {
            fullscreen: false,
            always_on_top: false,
            show_help,
            hex_input: String::new(),
            hex_input_color: None,
            color: Color32::from_hex(&settings.color).unwrap_or(Color32::BLACK),
            session_type: detect_session_type(),
            settings,
        }
    }

    // Help window: a list of controls, color picker, and close button
    fn show_help_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Help & Settings")
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
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.hex_input)
                            .desired_width(80.0)
                            .prefix("#"),
                    );
                    if response.changed() {
                        // Only hex digits are allowed: the leading '#' (which
                        // `from_hex` requires) is stripped for display and put back
                        // for parsing, and any other stray character is dropped.
                        // Trimming happens after stripping, so pasting "#rrggbb"
                        // keeps all six digits instead of losing one to the limit.
                        self.hex_input.retain(|c| c.is_ascii_hexdigit());
                        self.hex_input.truncate(6);
                        self.hex_input_color =
                            Color32::from_hex(&format!("#{}", self.hex_input)).ok();
                    }
                    // `valid`: the input parses to a colour; `changed`: that colour is not
                    // already the one in use. Apply makes sense only when both hold.
                    let valid = self.hex_input_color.is_some();
                    let changed = self.hex_input_color != Some(self.color);
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && valid {
                        if let Some(c) = self.hex_input_color {
                            self.color = c;
                            self.settings.color = c.to_hex();
                        }
                    }
                    if ui
                        .add_enabled(valid && changed, egui::Button::new("Apply"))
                        .clicked()
                    {
                        if let Some(c) = self.hex_input_color {
                            self.color = c;
                            self.settings.color = c.to_hex();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("or pick here:");
                    let mut srgb = [self.color.r(), self.color.g(), self.color.b()];
                    if ui.color_edit_button_srgb(&mut srgb).changed() {
                        let c = Color32::from_rgb(srgb[0], srgb[1], srgb[2]);
                        self.color = c;
                        self.settings.color = c.to_hex();
                        self.hex_input = c.to_hex()[1..7].to_owned();
                        self.hex_input_color = Some(c);
                    }
                });
                ui.separator();

                // --- Default window size ---
                ui.allocate_space(egui::vec2(0.0, 8.0));
                ui.horizontal(|ui| {
                    ui.label("Default window size:");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.width)
                            .range(MIN_WINDOW_WIDTH..=MAX_WINDOW_SIZE)
                            .suffix(" px"),
                    );
                    ui.label(" X ");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.height)
                            .range(MIN_WINDOW_HEIGHT..=MAX_WINDOW_SIZE)
                            .suffix(" px"),
                    );
                    ui.add_space(20.0);
                    if ui.button("Use current").clicked() {
                        let rect = ui.ctx().content_rect();
                        self.settings.width = rect.width().round() as u32;
                        self.settings.height = rect.height().round() as u32;
                    }
                    if ui.button(DEFAULT_SIZE_LABEL).clicked() {
                        self.settings.width = DEFAULT_WIDTH;
                        self.settings.height = DEFAULT_HEIGHT;
                    }
                });

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
                    ui.add_space(4.0);
                    ui.small(format!(" vers. {}", VERSION));
                });
            });
    }
}

impl eframe::App for BlackCurtain {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let color = self.color;
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

            // Double click
            if response.double_clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                self.fullscreen = !self.fullscreen;
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
            }

            // Right click
            if response.clicked_by(egui::PointerButton::Secondary)
                || (ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.show_help)
            {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }

            // L key
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

            // Middle click
            if response.clicked_by(egui::PointerButton::Middle)
                || ui.input(|i| i.key_pressed(egui::Key::F1))
            {
                self.show_help = true;
                self.hex_input = self.color.to_hex()[1..7].to_owned();
                self.hex_input_color = Some(self.color);
            }
        });

        if self.show_help {
            self.show_help_window(ui.ctx());
        }
    }
}
