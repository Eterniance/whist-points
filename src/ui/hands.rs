use egui::ModalResponse;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};
use whist::game::{
    GameError,
    contractors::Contractors,
    hand::{Hand, HandBuilder, HandRecap, InputError, InputRequest},
    players::{PlayerId, PlayerIdAndScore, Players},
    rules::Contract,
};

use crate::ui::requester::RequesterGui;

type IoResult<T> = Result<T, GameError>;

#[derive(Default, Deserialize, Serialize)]
pub struct HandBuilderGUI {
    pub players: Players,
    #[serde(skip)]
    pub hand_builder: Option<HandBuilder>,
    #[serde(skip)]
    requester: RequesterGui,
    #[serde(skip)]
    show_point_modal: bool,
}

impl HandBuilderGUI {
    pub fn new(players: Players) -> Self {
        Self {
            players,
            hand_builder: None,
            requester: RequesterGui::default(),
            show_point_modal: false,
        }
    }

    pub fn new_hand(&mut self, contract: Rc<Contract>) {
        self.hand_builder = Some(HandBuilder::new(contract));
        self.requester.clear();
    }

    fn get_next_id(&self, names: &mut indexmap::set::Iter<'_, String>) -> IoResult<PlayerId> {
        let id = self
            .players
            .get_id(
                names
                    .next()
                    .expect("Cannot call function without setting self.requester"),
            )
            .ok_or(InputError::InvalidInput("Player ID mismatch"))?;
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
            3 => {
                let mut out = Vec::new();
                for score in self
                    .requester
                    .points
                    .ok_or(InputError::InvalidInput("Points not set"))?
                {
                    out.push(PlayerIdAndScore::new(self.get_next_id(&mut names)?, score));
                }
                Ok(Contractors::Other(out))
            }
            _ => Err(GameError::TooManyPlayer),
        }
    }

    pub fn ui(
        &mut self,
        ui: &egui::Ui,
        players: &Players,
    ) -> IoResult<ModalResponse<Option<IoResult<Hand>>>> {
        if self.hand_builder.is_none() {
            return Err(InputError::InvalidInput("No contract set").into());
        }
        let mut requests = self
            .hand_builder
            .as_mut()
            .expect("Is not None")
            .all_requests()
            .into_iter();

        let resp = egui::Modal::new("new_hand".into()).show(ui.ctx(), |ui| {
            if self.show_point_modal {
                let order: HashMap<String, usize> = players
                    .names()
                    .into_iter()
                    .enumerate()
                    .map(|(i, n)| (n, i))
                    .collect();

                #[expect(clippy::indexing_slicing)]
                self.requester
                    .selected_names
                    .sort_by(|a, b| order[a].cmp(&order[b]));
                let resp = self.show_point_modal_ui(ui);
                if resp.should_close() {
                    self.show_point_modal = false;
                }
            }

            let n = match requests.next().expect("Always at least 1 element") {
                InputRequest::ContractorsSolo => 1,
                InputRequest::ContractorsTeam => 2,
                InputRequest::ContractorsOther => 3,
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
                        let contractors_number = self.requester.selected_names.len();
                        if contractors_number == 3 && self.requester.points.is_none() {
                            self.show_point_modal = true;
                            return None;
                        }
                        let hand_result: IoResult<Hand> = (|| {
                            let c = self.create_contractors(contractors_number)?;
                            let mut builder = self.hand_builder.take().expect("Is not None");
                            builder.set_contractors(c)?;
                            builder.set_bid(self.requester.bid_value)?;
                            builder.set_tricks(self.requester.tricks_value);
                            let hand = builder.build()?;
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

    fn show_point_modal_ui(&mut self, ui: &egui::Ui) -> ModalResponse<()> {
        if self.requester.points.is_none() {
            self.requester.points = Some([0, 0, 0]);
        }
        egui::Modal::new("points modal".into()).show(ui.ctx(), |ui| {
            if let Err(e) = self.requester.show_points(ui) {
                error!("error: {e}");
            }

            egui::Sides::new().show(
                ui,
                |_| {},
                |ui| {
                    if ui.button("Ok").clicked() {
                        ui.close();
                    }
                },
            );
        })
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct HandsHistoric {
    list: Vec<HandRecap>,
    players_scores: Vec<[i16; 4]>,
}

#[expect(clippy::indexing_slicing)]
impl HandsHistoric {
    pub fn show_hand(
        &self,
        ui: &egui::Ui,
        row_idx: usize,
        players: &[String],
    ) -> ModalResponse<()> {
        let hand = &self.list[row_idx];
        egui::Modal::new(format!("Hand {row_idx}").into()).show(ui.ctx(), |ui| {
            ui.label(format!("Mode: {}", hand.gamemode_name));
            ui.label(format!("Tricks: {}", hand.tricks));
            if let Some(bid) = hand.bid {
                ui.label(format!("Bid: {bid}"));
            }
            ui.separator();
            let vec_pias = hand.contractors.as_pias();
            ui.horizontal(|ui| {
                for pias in &vec_pias {
                    let (id, score) = pias.as_components();
                    let name = players[*id].clone();
                    ui.vertical(|ui| {
                        ui.label(name);
                        ui.label(format!("{score}"));
                    });
                }
            });

            egui::Sides::new().show(
                ui,
                |_| {},
                |ui| {
                    if ui.button("Ok").clicked() {
                        ui.close();
                    }
                },
            );
        })
    }

    pub fn push(&mut self, hand_recap: HandRecap) {
        if let Some(scores) = self.players_scores.last() {
            let new_scores: [i16; 4] = std::array::from_fn(|i| scores[i] + hand_recap.scores[i]);
            self.players_scores.push(new_scores);
        } else {
            self.players_scores.push(hand_recap.scores);
        }
        self.list.push(hand_recap);
    }

    pub fn len(&self) -> usize {
        assert_eq!(
            self.list.len(),
            self.players_scores.len(),
            "Length difference would imply a misuse of the struct"
        );
        self.list.len()
    }

    pub fn remove_last(&mut self) {
        assert_eq!(
            self.list.len(),
            self.players_scores.len(),
            "Length difference would imply a misuse of the struct"
        );
        self.list.pop();
        self.players_scores.pop();
    }
}

impl<'a> IntoIterator for &'a HandsHistoric {
    type Item = (&'a HandRecap, &'a [i16; 4]);
    type IntoIter = std::iter::Zip<std::slice::Iter<'a, HandRecap>, std::slice::Iter<'a, [i16; 4]>>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.iter().zip(self.players_scores.iter())
    }
}
