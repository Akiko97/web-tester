#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::Frame;
use egui::Context;
use egui::Image;
use poll_promise::Promise;
use std::collections::HashMap;
use egui_extras::{Column, TableBuilder};

struct Resource {
    response: ehttp::Response,
    text: Option<String>,
    image: Option<Image<'static>>,
}

impl Resource {
    fn from_response(ctx: &Context, response: ehttp::Response) -> Self {
        let content_type = response.content_type().unwrap_or_default();
        if content_type.starts_with("image/") {
            ctx.include_bytes(response.url.clone(), response.bytes.clone());
            let image = Image::from_uri(response.url.clone());
            Self {
                response,
                text: None,
                image: Some(image),
            }
        } else {
            let text = response.text();
            let text = text.map(|text| text.to_owned());
            Self {
                response,
                text,
                image: None,
            }
        }
    }
}

#[derive(PartialEq)]
enum RequireType {
    GET, POST
}

#[derive(PartialEq)]
enum POSTType {
    Form, JSON
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct APP {
    url: String,
    require_type: RequireType,
    post_type: POSTType,
    use_param_input: bool,
    params: HashMap<String, String>,
    params_vec: Vec<String>,
    field: String,
    value: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    promise: Option<Promise<ehttp::Result<Resource>>>,
}

impl Default for APP {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8080/get".to_owned(),
            require_type: RequireType::GET,
            post_type: POSTType::JSON,
            use_param_input: false,
            field: "".to_owned(),
            value: "".to_owned(),
            params: HashMap::new(),
            params_vec: vec![],
            promise: Default::default(),
        }
    }
}

impl eframe::App for APP {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Web Tester");
            ui.end_row();
            ui.horizontal(|ui| {
                ui.label("URL: ");
                ui.add(egui::TextEdit::singleline(&mut self.url).desired_width(f32::INFINITY)).lost_focus();
            });
            ui.end_row();
            ui.horizontal(|ui| {
                ui.label("Require Type: ");
                ui.selectable_value(&mut self.require_type, RequireType::GET, "GET");
                ui.selectable_value(&mut self.require_type, RequireType::POST, "POST");
            });
            ui.end_row();
            if self.require_type == RequireType::GET {
                ui.checkbox(&mut self.use_param_input, "Use parameter input");
                ui.end_row();
            }
            if self.require_type == RequireType::POST {
                ui.horizontal(|ui| {
                    ui.label("POST Type: ");
                    ui.selectable_value(&mut self.post_type, POSTType::Form, "Form");
                    ui.selectable_value(&mut self.post_type, POSTType::JSON, "JSON");
                });
                ui.end_row();
            }
            if (self.require_type == RequireType::GET && self.use_param_input) ||
                self.require_type == RequireType::POST {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Field: ");
                        ui.add(egui::TextEdit::singleline(&mut self.field).desired_width(120f32)).lost_focus();
                        ui.label("Value: ");
                        ui.add(egui::TextEdit::singleline(&mut self.value).desired_width(120f32)).lost_focus();
                    });
                    ui.end_row();
                    ui.horizontal(|ui| {
                        if ui.button("ADD").clicked() {
                            if !self.params_vec.contains(&self.field) {
                                self.params_vec.push(self.field.clone());
                            }
                            self.params.insert(self.field.clone(), self.value.clone());
                            self.field = "".to_owned();
                            self.value = "".to_owned();
                        }
                    });
                    ui.end_row();
                    let text_height = egui::TextStyle::Body
                        .resolve(ui.style())
                        .size
                        .max(ui.spacing().interact_size.y);
                    let mut table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::initial(150.0).at_least(150.0))
                        .column(Column::initial(150.0).at_least(150.0))
                        .column(Column::initial(150.0).at_least(150.0))
                        .min_scrolled_height(0.0);
                    let mut need_delete: Option<usize> = None;
                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Field");
                            });
                            header.col(|ui| {
                                ui.strong("Value");
                            });
                            header.col(|ui| {
                                ui.strong("Edit");
                            });
                        })
                        .body(|mut body| {
                            body.rows(text_height, self.params.len(), |mut row| {
                                let row_index = row.index();
                                row.col(|ui| {
                                    ui.label(self.params_vec[row_index].clone());
                                });
                                row.col(|ui| {
                                    ui.label(self.params.get(&self.params_vec[row_index]).unwrap().clone());
                                });
                                row.col(|ui| {
                                    if ui.button("DELETE").clicked() {
                                        need_delete = Option::from(row_index);
                                    }
                                    if ui.button("EDIT").clicked() {
                                        self.field = self.params_vec[row_index].clone();
                                        self.value = self.params.get(&self.params_vec[row_index]).unwrap().clone();
                                    }
                                });
                            });
                        });
                    if let Some(row_index) = need_delete {
                        self.params.remove(&self.params_vec[row_index]);
                        self.params_vec.remove(row_index);
                    }
                });
            }
            if ui.button("Require").clicked() {
                let ctx = ctx.clone();
                let (sender, promise) = Promise::new();
                match self.require_type {
                    RequireType::GET => {
                        let mut url = self.url.clone();
                        if self.use_param_input {
                            url += "?";
                            self.params_vec.iter().for_each(|field| {
                                url += &*field.clone();
                                url += "=";
                                url += &*self.params.get(&field.clone()).unwrap().clone();
                                url += "&";
                            });
                            url.pop();
                        }
                        let request = ehttp::Request::get(&url);
                        ehttp::fetch(request, move |response| {
                            ctx.forget_image(&url);
                            ctx.request_repaint();
                            let resource = response.map(|response| Resource::from_response(&ctx, response));
                            sender.send(resource);
                        });
                    }
                    RequireType::POST => {
                        let url = self.url.clone();
                        let mut body = String::new();
                        match self.post_type {
                            POSTType::Form => {
                                self.params_vec.iter().for_each(|field| {
                                    body += &*field.clone();
                                    body += "=";
                                    body += &*self.params.get(&field.clone()).unwrap().clone();
                                    body += "&";
                                });
                                body.pop();
                            }
                            POSTType::JSON => {
                                body = serde_json::to_string(&self.params).unwrap();
                                println!("{body}");
                            }
                        }
                        let mut request = ehttp::Request::post(&url, Vec::from(body.as_bytes()));
                        match self.post_type {
                            POSTType::Form => {
                                request.headers.insert("Content-Type", "application/x-www-form-urlencoded");
                            }
                            POSTType::JSON => {
                                request.headers.insert("Content-Type", "application/json");
                            }
                        }
                        ehttp::fetch(request, move |response| {
                            ctx.forget_image(&url);
                            ctx.request_repaint();
                            let resource = response.map(|response| Resource::from_response(&ctx, response));
                            sender.send(resource);
                        });
                    }
                }
                self.promise = Some(promise);
            }
            ui.separator();
            if let Some(promise) = &self.promise {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(resource) => {
                            let Resource {
                                response,
                                text,
                                image,
                            } = resource;
                            ui.monospace(format!("url:          {}", response.url));
                            ui.monospace(format!(
                                "status:       {} ({})",
                                response.status, response.status_text
                            ));
                            ui.monospace(format!(
                                "content-type: {}",
                                response.content_type().unwrap_or_default()
                            ));
                            ui.monospace(format!(
                                "size:         {:.1} kB",
                                response.bytes.len() as f32 / 1000.0
                            ));
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .auto_shrink(false)
                                .show(ui, |ui| {
                                    egui::CollapsingHeader::new("Response headers")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            egui::Grid::new("response_headers")
                                                .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                                                .show(ui, |ui| {
                                                    for (k, v) in &response.headers {
                                                        ui.label(k);
                                                        ui.label(v);
                                                        ui.end_row();
                                                    }
                                                })
                                        });
                                    ui.separator();
                                    if let Some(text) = &text {
                                        let tooltip = "Click to copy the response body";
                                        if ui.button("📋").on_hover_text(tooltip).clicked() {
                                            ui.ctx().copy_text(text.clone());
                                        }
                                        ui.separator();
                                    }
                                    if let Some(image) = image {
                                        ui.add(image.clone());
                                    } else if let Some(text) = &text {
                                        ui.add(egui::Label::new(text));
                                    } else {
                                        ui.monospace("[binary]");
                                    }
                                });
                        }
                        Err(error) => {
                            ui.colored_label(
                                ui.visuals().error_fg_color,
                                if error.is_empty() { "Error" } else { error },
                            );
                        }
                    }
                } else {
                    ui.spinner();
                }
            }
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
