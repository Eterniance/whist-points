use egui::ModalResponse;
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use whist::game::{
    GameError,
    hand::{Hand, HandBuilder, HandRecap, InputError, InputRequest},
    players::{Contractors, PlayerId, Players},
    rules::Contract,
};

use crate::whist::requester::RequesterGui;

type IoResult<T> = Result<T, GameError>;

#[derive(Default, Deserialize, Serialize)]
pub struct HandBuilderGUI {
    pub players: Players,
    #[serde(skip)]
    pub hand_builder: Option<HandBuilder>,
    #[serde(skip)]
    requester: RequesterGui,
}

impl HandBuilderGUI {
    pub fn new(players: Players) -> Self {
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
    ) -> IoResult<PlayerId> {
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

    fn create_contractors(&self, contractors_number: usize) -> IoResult<Contractors> {
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

    pub fn ui(
        &mut self,
        ui: &egui::Ui,
        players: &Players,
    ) -> IoResult<ModalResponse<Option<IoResult<Hand>>>> {
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
            let ready = self.requester.show_names(ui, &players.names(), n);

            if let Some(InputRequest::Bid { min, max }) = requests.next() {
                ui.separator();
                self.requester.show_bid(ui, min..=max);
            }

            ui.separator();
            self.requester.show_tricks(ui);

            let (_, hand_res) = egui::Sides::new().show(
                ui,
                |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                        // return Err(GameError::HandBuildError("Cancel".to_string()));
                    }
                },
                |ui| {
                    if ui.add_enabled(ready, egui::Button::new("Ok")).clicked() {
                        let hand_result: IoResult<Hand> = (|| {
                            let c = self.create_contractors(n)?;
                            let mut builder = self.hand_builder.take().expect("Is not None");
                            builder.set_contractors(c)?;
                            builder.set_bid(self.requester.bid_value)?;
                            builder.set_tricks(self.requester.tricks_value);
                            let hand = builder.build()?;
                            debug!("hand {hand:#?}");
                            Ok(hand)
                        })();

                        if hand_result.is_ok() {
                            ui.close();
                        }
                        return Some(hand_result);
                    }
                    None
                },
            );
            hand_res
        });
        debug!("resp : {:?}", resp.inner);
        Ok(resp)
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct HandsHistoric {
    list: Vec<HandRecap>,
}
