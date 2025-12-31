use egui::{ModalResponse, ahash::HashSet};
use serde::{Deserialize, Serialize};
use std::{ops::RangeInclusive, sync::Arc};
use whist::game::{
    hand::{HandBuilder, InputError, InputRequest, Requester},
    players::Players,
    rules::Contract,
};

#[derive(Default, Deserialize, Serialize)]
pub struct HandBuilderGUI {
    pub players: Arc<Players>,
    #[serde(skip)]
    pub hand_builder: Option<HandBuilder>,
    requester: RequesterGui,
}

impl HandBuilderGUI {
    pub fn new(players: Arc<Players>) -> Self {
        Self {
            players,
            hand_builder: None,
            requester: RequesterGui::default(),
        }
    }

    pub fn new_hand(&mut self, contract: Arc<Contract>) {
        self.hand_builder = Some(HandBuilder::new(contract));
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Result<ModalResponse<()>, &'static str> {
        if self.hand_builder.is_none() {
            return Err("No contract set");
        }
        let b = self.hand_builder.as_mut().expect("Is not None");
        let mut requests = b.all_requests().into_iter();

        let resp = egui::Modal::new("new_hand".into()).show(ui.ctx(), |ui| {
            let n = match requests.next().expect("Always at least 1 element") {
                InputRequest::ContractorsSolo => 1,
                InputRequest::ContractorsTeam => 2,
                InputRequest::ContractorsOther => 4,
                _ => unreachable!(),
            };
            self.requester.show_names(ui, &self.players.names(), n);

            egui::Sides::new().show(
                ui,
                |_ui| {},
                |ui| {
                    if ui.button("Ok").clicked() {
                        ui.close();
                    }
                },
            );
        });
        Ok(resp)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RequesterGui {
    selected_names: HashSet<String>,
}

impl RequesterGui {
    pub fn show_names(&mut self, ui: &mut egui::Ui, names: &[String], n: usize) {
        let selected_count = self.selected_names.len();

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
            }
        }
    }
}
