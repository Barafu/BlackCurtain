use std::time::Duration;

use eframe::egui::{self, Color32};

fn main() -> eframe::Result {
    let color = Color32::BLACK;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_title("Black Curtain")
            .with_app_id("black_curtain"),
        ..Default::default()
    };

    eframe::run_native(
        "Black Curtain",
        options,
        Box::new(move |_cc| Ok(Box::new(BlackCurtain::new(color)))),
    )
}

fn parse_hex_color(hex: &str) -> Result<Color32, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 3 && hex.len() != 6 {
        return Err("expected 3 or 6 hex digits".into());
    }
    let hex = if hex.len() == 3 {
        format!(
            "{}{}{}{}{}{}",
            &hex[0..1], &hex[0..1],
            &hex[1..2], &hex[1..2],
            &hex[2..3], &hex[2..3],
        )
    } else {
        hex.to_string()
    };
    let val = u32::from_str_radix(&hex, 16).map_err(|_| format!("invalid hex '{}'", hex))?;
    Ok(Color32::from_rgb(
        ((val >> 16) & 0xFF) as u8,
        ((val >> 8) & 0xFF) as u8,
        (val & 0xFF) as u8,
    ))
}

/// Application state.
struct BlackCurtain {
    fullscreen: bool,
    show_help: bool,
    color: Color32,
    hex_input: String,
    hex_valid: bool,
}

impl BlackCurtain {
    fn new(color: Color32) -> Self {
        Self {
            fullscreen: false,
            show_help: false,
            color,
            hex_input: String::new(),
            hex_valid: false,
        }
    }

    // Help window: controls table, color picker, and close button
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
                    });

                // --- Color input row ---
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.add(egui::TextEdit::singleline(&mut self.hex_input).char_limit(7).desired_width(80.0));
                    let valid = parse_hex_color(&self.hex_input).is_ok();
                    self.hex_valid = valid;
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && valid {
                        if let Ok(c) = parse_hex_color(&self.hex_input) {
                            self.color = c;
                        }
                    }
                    if ui.add_enabled(valid, egui::Button::new("Apply")).clicked() {
                        if let Ok(c) = parse_hex_color(&self.hex_input) {
                            self.color = c;
                        }
                    }
                    ui.label("or pick:");
                    let mut srgb = [self.color.r(), self.color.g(), self.color.b()];
                    if ui.color_edit_button_srgb(&mut srgb).changed() {
                        self.color = Color32::from_rgb(srgb[0], srgb[1], srgb[2]);
                        self.hex_input = format!("#{:02x}{:02x}{:02x}", srgb[0], srgb[1], srgb[2]);
                    }
                });

                // --- Close button ---
                ui.allocate_space(egui::vec2(0.0, 8.0));
                ui.vertical_centered(|ui| {
                    if ui.add_sized(egui::vec2(120.0, 32.0), egui::Button::new("Close")).clicked() {
                        self.show_help = false;
                    }
                });
            });
    }
}

impl eframe::App for BlackCurtain {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let color = self.color;
        ui.ctx().global_style_mut(move |style| {
            style.visuals.panel_fill = color;
            style.visuals.window_fill = color;
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

            if response.double_clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Space))
            {
                self.fullscreen = !self.fullscreen;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
            }
            if response.clicked_by(egui::PointerButton::Secondary)
                || (ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.show_help)
            {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            if response.clicked_by(egui::PointerButton::Middle)
                || ui.input(|i| i.key_pressed(egui::Key::F1))
            {
                self.show_help = !self.show_help;
                if self.show_help {
                    let r = self.color.r();
                    let g = self.color.g();
                    let b = self.color.b();
                    self.hex_input = format!("#{r:02x}{g:02x}{b:02x}");
                }
            }
        });

        if self.show_help {
            self.show_help_window(ui.ctx());
        }
    }
}
