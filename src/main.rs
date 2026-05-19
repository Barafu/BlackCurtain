//! Fills a monitor with a solid color. Acts as a "curtain" that blacks out or
//! dims a screen. Supports optional hex color argument, fullscreen toggle on
//! double-click, and minimized on right-click.
//!
//! CLI arguments:
//! - `<hex_color>` — start with the given color (e.g. `#ff0000`)

use eframe::egui::{self, Color32};

fn main() -> eframe::Result {
    // Parse optional hex color from first positional argument
    let color = std::env::args()
        .nth(1)
        .as_deref()
        .map(parse_hex_color)
        .unwrap_or(Color32::BLACK);

    // Configure a minimal window with no chrome overrides
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

/// Parses an HTML-style hex color string (e.g. `#ff0000` or `#f00`) into a
/// [`Color32`]. Falls back to black on invalid input.
fn parse_hex_color(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.is_empty() {
        return Color32::BLACK;
    }
    // Expand 3-digit shorthand (e.g. #f00 → #ff0000)
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
    let Ok(val) = u32::from_str_radix(&hex, 16) else {
        eprintln!("warning: invalid hex color '{}', falling back to black", hex);
        return Color32::BLACK;
    };
    Color32::from_rgb(
        ((val >> 16) & 0xFF) as u8,
        ((val >> 8) & 0xFF) as u8,
        (val & 0xFF) as u8,
    )
}

/// Application state.
struct BlackCurtain {
    fullscreen: bool,
    color: Color32,
}

impl BlackCurtain {
    fn new(color: Color32) -> Self {
        Self {
            fullscreen: false,
            color,
        }
    }
}

impl eframe::App for BlackCurtain {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let color = self.color;
        ui.ctx().global_style_mut(move |style| {
            style.visuals.panel_fill = color;
            style.visuals.window_fill = color;
        });

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
                || ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        });
    }
}
