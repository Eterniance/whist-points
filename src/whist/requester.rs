use egui::ahash::HashSet;
use log::debug;
use std::ops::RangeInclusive;

#[derive(Debug, Default)]
pub struct RequesterGui {
    pub selected_names: HashSet<String>,
    pub bid_value: i16,
    pub tricks_value: i16,
}

impl RequesterGui {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn show_names(&mut self, ui: &mut egui::Ui, names: &[String], n: usize) -> bool {
        let selected_count = self.selected_names.len();

        ui.label("Select contractors");
        ui.separator();
        for name in names {
            let is_selected = self.selected_names.contains(name);
            let can_select_more = selected_count < n || is_selected;

            let resp = ui.add_enabled(can_select_more, egui::Button::selectable(is_selected, name));

            if resp.clicked() {
                if is_selected {
                    self.selected_names.remove(name);
                } else {
                    self.selected_names.insert(name.clone());
                }
                debug!("{:?}", self.selected_names);
            }
        }
        selected_count == n
    }

    pub fn show_bid(&mut self, ui: &mut egui::Ui, range: RangeInclusive<i16>) {
        ui.horizontal(|ui| {
            ui.label("Tricks to win ?");
            ui.add(egui::DragValue::new(&mut self.bid_value).range(range));
        });
    }

    pub fn show_tricks(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Tricks number");
            ui.add(egui::DragValue::new(&mut self.tricks_value).range(0..=13));
        });
    }
}
