use eframe::egui;
use serde::{Deserialize, Serialize};
use crate::save;

pub struct StarEditor {
    selected: Option<usize>,
    objects: Vec<GameObject>,
    zoom: f32,
    dragging: Option<usize>,
    drag_start: Option<egui::Pos2>,
    view_offset: [f32; 2],
    pan_start: Option<egui::Pos2>,
    image_cache: std::collections::HashMap<String, egui::TextureHandle>,
}

impl Default for StarEditor {
    fn default() -> Self {
        Self {
            selected: None,
            objects: vec![],
            zoom: 1.0,
            dragging: None,
            drag_start: None,
            view_offset: [0.0, 0.0],
            pan_start: None,
            image_cache: std::collections::HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameObject {
    id: usize,
    name: String,
    position: [f32; 2],
    rotation: f32,
    scale: [f32; 2],
    pub image_path: Option<String>,
}

impl StarEditor {
    pub fn load_image(path: &str, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        use image::io::Reader as ImageReader;
        use image::GenericImageView;

        let reader = ImageReader::open(path).ok()?;
        let img = reader.decode().ok()?;
        let size = [img.width() as usize, img.height() as usize];
        let rgba = img.to_rgba8();
        let pixels = rgba.as_flat_samples();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        Some(ctx.load_texture(path.to_string(), color_image, Default::default()))
    }
}

impl eframe::App for StarEditor {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("hierarchy").show(ctx, |ui| {
            ui.heading("Hierarchy");
            for (i, obj) in self.objects.iter().enumerate() {
                if ui.selectable_label(self.selected == Some(i), &obj.name).clicked() {
                    self.selected = Some(i);
                }
            }
            if ui.button("Add Object").clicked() {
                let id = self.objects.len();
                self.objects.push(GameObject {
                    id,
                    name: format!("Object {}", id),
                    position: [0.0, 0.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                    image_path: None,
                });
            }
            ui.separator();
            if ui.button("💾 Save Scene").clicked() {
                save::save_scene(&self.objects, "scene.ron");
            }
            if ui.button("📂 Load Scene").clicked() {
                self.objects = save::load_scene("scene.ron");
            }
        });

        egui::SidePanel::right("inspector").show(ctx, |ui| {
            ui.heading("Inspector");
            if let Some(i) = self.selected {
                let path = self.objects[i].image_path.clone().unwrap_or_default();

                if !self.image_cache.contains_key(&path) {
                    if let Some(tex) = StarEditor::load_image(&path, ctx) {
                        self.image_cache.insert(path.clone(), tex);
                    }
                }

                let obj = &mut self.objects[i];
                obj.image_path = Some(path.clone());

                ui.label(format!("ID: {}", obj.id));
                ui.text_edit_singleline(&mut obj.name);
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add(egui::DragValue::new(&mut obj.position[0]));
                    ui.add(egui::DragValue::new(&mut obj.position[1]));
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.add(egui::DragValue::new(&mut obj.rotation));
                });
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    ui.add(egui::DragValue::new(&mut obj.scale[0]));
                    ui.add(egui::DragValue::new(&mut obj.scale[1]));
                });
            } else {
                ui.label("No object selected.");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // zoom 조절
            let zoom_delta = ctx.input(|i| {
                i.events.iter().filter_map(|e| match e {
                    egui::Event::Scroll(delta) => Some(delta.y),
                    _ => None,
                }).sum::<f32>()
            });
            if zoom_delta != 0.0 {
                self.zoom += zoom_delta * 0.01;
                self.zoom = self.zoom.clamp(0.1, 5.0);
            }

            ui.heading("Scene View");

            // 영역 확보 및 상호작용 등록
            let available_size = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));

            let pointer_pos = response.interact_pointer_pos();

            for (i, obj) in self.objects.iter_mut().enumerate() {
                let center = egui::pos2(
                    rect.left_top().x + self.view_offset[0] + obj.position[0] * 10.0 * self.zoom,
                    rect.left_top().y + self.view_offset[1] + obj.position[1] * 10.0 * self.zoom,
                );

                let size_x = obj.scale[0] * 20.0 * self.zoom;
                let size_y = obj.scale[1] * 20.0 * self.zoom;

                let bounding = egui::Rect::from_center_size(center, egui::vec2(size_x, size_y));

                // 클릭 시작
                if response.drag_started() {
                    if let Some(pos) = pointer_pos {
                        if bounding.contains(pos) {
                            self.dragging = Some(i);
                            self.drag_start = Some(pos);
                            self.selected = Some(i);
                        }
                    }
                }

                // 드래그 중
                if self.dragging == Some(i) {
                    if let (Some(pos), Some(start)) = (pointer_pos, self.drag_start) {
                        let delta = pos - start;
                        obj.position[0] += delta.x / (10.0 * self.zoom);
                        obj.position[1] += delta.y / (10.0 * self.zoom);
                        self.drag_start = Some(pos);
                    }

                    // 마우스 뗐을 때
                    if response.drag_released() {
                        self.dragging = None;
                        self.drag_start = None;
                    }
                }

                // 오브젝트 그리기
                let angle = obj.rotation;
                let half_w = size_x / 2.0;
                let half_h = size_y / 2.0;
                let points = [
                    (-half_w, -half_h),
                    (half_w, -half_h),
                    (half_w, half_h),
                    (-half_w, half_h),
                ];
                let rotated: Vec<egui::Pos2> = points
                    .iter()
                    .map(|(dx, dy)| {
                        let rx = dx * angle.cos() - dy * angle.sin();
                        let ry = dx * angle.sin() + dy * angle.cos();
                        egui::pos2(center.x + rx, center.y + ry)
                    })
                    .collect();

                if ctx.input(|i| i.pointer.secondary_down()) {
                    if let Some(current) = response.interact_pointer_pos() {
                        if let Some(start) = self.pan_start {
                            let delta = current - start;
                            self.view_offset[0] += delta.x;
                            self.view_offset[1] += delta.y;
                            self.pan_start = Some(current);
                        } else {
                            self.pan_start = Some(current);
                        }
                    }
                } else {
                    self.pan_start = None;
                }

                if let Some(path) = &obj.image_path {
                    if let Some(tex) = self.image_cache.get(path) {
                        let size = egui::vec2(size_x, size_y);
                        let pos = egui::pos2(center.x - size_x / 2.0, center.y - size_y / 2.0);
                        painter.image(
                            tex.id(),
                            egui::Rect::from_min_size(pos, size),
                            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                        continue;
                    }
                }

                ctx.input(|i| {
                    let step = 10.0;
                    if i.key_down(egui::Key::W) {
                        self.view_offset[1] += step;
                    }
                    if i.key_down(egui::Key::S) {
                        self.view_offset[1] -= step;
                    }
                    if i.key_down(egui::Key::A) {
                        self.view_offset[0] += step;
                    }
                    if i.key_down(egui::Key::D) {
                        self.view_offset[0] -= step;
                    }
                });

                let stroke_color = if self.selected == Some(i) {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::LIGHT_BLUE
                };

                painter.add(egui::Shape::closed_line(
                    rotated,
                    egui::Stroke::new(2.0, stroke_color),
                ));

                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    &obj.name,
                    egui::FontId::monospace(10.0),
                    egui::Color32::WHITE,
                );
            }
        });
    }
}