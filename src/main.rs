#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::Frame;
use egui::Context;

struct APP {
    value: String,
}

impl Default for APP {
    fn default() -> Self {
        Self {
            value: String::from("Akiko"),
        }
    }
}

impl eframe::App for APP {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{}", self.value.clone()));
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200f32, 800f32]),
        ..Default::default()
    };
    eframe::run_native(
        "Web Tester",
        options,
        Box::new(|_cc| Box::new(APP::default())),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|_cc| Box::new(APP::default())),
            )
            .await
            .expect("failed to start eframe");
    });
}
