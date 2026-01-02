use egui::{ModalResponse, ahash::HashSet};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{ops::RangeInclusive, sync::Arc};
use whist::game::{
    GameError,
    hand::{HandBuilder, InputError, InputRequest},
    players::{Contractors, PlayerId, Players},
    rules::Contract,
};

type Wesult<T> = Result<T, GameError>;

#[derive(Default, Deserialize, Serialize)]
pub struct HandBuilderGUI {
    pub players: Arc<Players>,
    #[serde(skip)]
    pub hand_builder: Option<HandBuilder>,
    #[serde(skip)]
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
        self.requester.clear();
    }

    fn get_next_id(
        &self,
        names: &mut std::collections::hash_set::Iter<'_, String>,
    ) -> Wesult<PlayerId> {
        let id = self
            .players
            .get_id(
                names
                    .next()
                    .expect("Cannot call function without setting self.requester"),
            )
            .ok_or(InputError::InvalidInput("Player ID mismatch".to_owned()))?;
        Ok(id)
    }

    fn create_contractors(&self, contractors_number: usize) -> Wesult<Contractors> {
        let mut names = self.requester.selected_names.iter();
        match contractors_number {
            1 => {
                let id = self.get_next_id(&mut names)?;
                Ok(Contractors::Solo(id))
            }
            2 => {
                let id1 = self.get_next_id(&mut names)?;
                let id2 = self.get_next_id(&mut names)?;
                Ok(Contractors::Team(id1, id2))
            }
            4 => Ok(Contractors::Other),
            _ => unreachable!(),
        }
    }

    pub fn ui(&mut self, ui: &egui::Ui) -> Wesult<ModalResponse<()>> {
        if self.hand_builder.is_none() {
            return Err(InputError::InvalidInput("No contract set".to_owned()).into());
        }
        let mut requests = self
            .hand_builder
            .as_mut()
            .expect("Is not None")
            .all_requests()
            .into_iter();

        let resp = egui::Modal::new("new_hand".into()).show(ui.ctx(), |ui| {
            let n = match requests.next().expect("Always at least 1 element") {
                InputRequest::ContractorsSolo => 1,
                InputRequest::ContractorsTeam => 2,
                InputRequest::ContractorsOther => 4,
                _ => unreachable!(),
            };
            let ready = self.requester.show_names(ui, &self.players.names(), n);

            if let Some(InputRequest::Bid { min, max }) = requests.next() {
                ui.separator();
                self.requester.show_bid(ui, min..=max);
            }

            ui.separator();
            self.requester.show_tricks(ui);

            let (_, _e) = egui::Sides::new().show(
                ui,
                |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                },
                |ui| {
                    if ui.add_enabled(ready, egui::Button::new("Ok")).clicked() {
                        let c = match self.create_contractors(n) {
                            Ok(c) => c,
                            Err(e) => return Err(e),
                        };
                        let builder = self.hand_builder.as_mut().expect("Is not None");
                        builder.set_contractors(c)?;
                        builder.set_bid(self.requester.bid_value)?;

                        debug!("{builder:#?}");
                        ui.close();
                    }
                    Ok(())
                },
            );
        });
        Ok(resp)
    }
}

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
