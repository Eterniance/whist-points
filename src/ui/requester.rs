use indexmap::IndexSet;
use log::debug;
use std::ops::RangeInclusive;

use crate::ui::AppError;

#[derive(Debug, Default)]
pub struct RequesterGui {
    pub selected_names: IndexSet<String>,
    pub bid_value: i16,
    pub tricks_value: i16,
    pub points: Option<[i16; 3]>,
}

impl RequesterGui {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn show_names(&mut self, ui: &mut egui::Ui, names: &[String], n: usize) -> bool {
        let selected_count = self.selected_names.len();
        let size = egui::vec2(ui.max_rect().size().x, 1.);
        // let min_size = dbg!(egui::vec2(186.0, ui.style().spacing.button_padding.y));
        ui.label("Select contractors");
        ui.separator();
        for name in names {
            let is_selected = self.selected_names.contains(name);
            let can_select_more = selected_count < n || is_selected;

            let resp = ui
                .add_enabled_ui(can_select_more, |ui| {
                    ui.add_sized(size, egui::Button::selectable(is_selected, name))
                })
                .inner;

            if resp.clicked() {
                if is_selected {
                    self.selected_names.shift_remove(name);
                } else {
                    self.selected_names.insert(name.clone());
                }
                debug!("{:?}", self.selected_names);
            }
        }
        if n == 3 {
            selected_count > 0
        } else {
            selected_count == n
        }
    }

    pub fn show_bid(&mut self, ui: &mut egui::Ui, range: RangeInclusive<i16>) {
        ui.horizontal(|ui| {
            ui.label("Tricks to win ?");
            ui.add(
                egui::DragValue::new(&mut self.bid_value)
                    .range(range)
                    .speed(0.05),
            );
        });
    }

    pub fn show_tricks(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Tricks number");
            ui.add(
                egui::DragValue::new(&mut self.tricks_value)
                    .range(0..=13)
                    .speed(0.05),
            );
        });
    }

    pub fn show_points(&mut self, ui: &mut egui::Ui) -> Result<(), AppError> {
        if self.selected_names.len() != 3 {
            return Err(AppError::ImpossibleState(format!(
                "Input point Ui is created for {}",
                self.selected_names.len()
            )));
        }
        let points = self.points.as_mut().expect("Should not be None");
        ui.label("Custom points input");
        for (idx, name) in self.selected_names.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(name);
                ui.add(
                    egui::DragValue::new(points.get_mut(idx).expect("both array matches length"))
                        .range(-240..=240)
                        .speed(0.1),
                );
            });
        }
        Ok(())
    }
}
