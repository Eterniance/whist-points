use crate::ui::requester::RequesterGui;
use egui::ModalResponse;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use whist_game::{
    Contract, GameError, Hand, HandRecap, InputError, PlayerId, Players, hand::HandBuilder,
    hand::InputRequest,
};
use whist_game::{HandBuildError, Tricks};

type IoResult<T> = Result<T, GameError>;

pub enum PendingHand {
    Classical(IoResult<Hand>),
    Custom,
}

#[derive(Deserialize, Serialize)]
pub struct HandBuilderGUI {
    pub players: Players,
    #[serde(skip)]
    pub hand_builder: Option<HandBuilder>,
    #[serde(skip)]
    pub requester: RequesterGui,
    #[serde(skip)]
    show_point_modal: bool,
    #[serde(skip)]
    custom_points_mode: bool,
}

impl HandBuilderGUI {
    pub fn new(players: Players) -> Self {
        Self {
            players,
            hand_builder: None,
            requester: RequesterGui::default(),
            show_point_modal: false,
            custom_points_mode: false,
        }
    }

    pub fn new_hand(&mut self, contract: Contract) {
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

    pub fn custom_hand_recap(&mut self) -> IoResult<HandRecap> {
        let contractors = self.create_contractors(3)?;
        let gamemode_name = self
            .hand_builder
            .take()
            .ok_or(HandBuildError("No contract set"))?
            .contract_name();
        let mut scores = [0; 4];
        let points = self
            .requester
            .points
            .ok_or(HandBuildError("No custom points"))?;
        let mut remaining_score = 0;
        let mut remaining_idx: usize = 6;
        for (i, id) in contractors.iter().enumerate() {
            let point = *points.get(i).expect("Withing range");
            *scores.get_mut(id.idx()).expect("Withing range") = point;
            remaining_score -= point;
            remaining_idx -= id.idx();
        }
        *scores.get_mut(remaining_idx).expect("Withing range") = remaining_score;
        let hand_recap: HandRecap = HandRecap {
            scores,
            gamemode_name,
            contractors_tricks: contractors
                .iter()
                .map(|id| (*id, Tricks::new(0).expect("Withing range")))
                .collect(),
            bid: None,
        };
        self.requester.clear();
        Ok(hand_recap)
    }

    fn create_contractors(&self, contractors_number: usize) -> IoResult<Vec<PlayerId>> {
        if self.requester.selected_names.len() != contractors_number {
            return Err(InputError::InvalidInput("Incorrect players number").into());
        }
        let mut contractors = vec![];
        let mut names = self.requester.selected_names.iter();
        for _ in &self.requester.selected_names {
            let id = self.get_next_id(&mut names)?;
            contractors.push(id);
        }
        Ok(contractors)
    }

    pub fn ui(
        &mut self,
        ui: &egui::Ui,
        players: &Players,
    ) -> IoResult<ModalResponse<Option<PendingHand>>> {
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
                self.custom_points_mode = resp.inner;
                if resp.should_close() {
                    self.show_point_modal = false;
                }
            }

            let n = match requests.next().expect("Always at least 1 element") {
                InputRequest::PlayersNumber { min, max } => min..=max,
                _ => unreachable!(),
            };
            let ready = self.requester.show_names(ui, &players.names(), n);

            if let Some(InputRequest::Bid { min, max }) = requests.next() {
                ui.separator();
                self.requester.show_bid(ui, min.get()..=max.get());
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
                        } else if self.custom_points_mode {
                            self.custom_points_mode = false;
                            return Some(PendingHand::Custom);
                        } else {
                            let hand_result: IoResult<Hand> = (|| {
                                let c = self.create_contractors(contractors_number)?;
                                let mut builder = self.hand_builder.take().expect("Is not None");
                                builder.set_contractors(&c)?;
                                builder.set_bid(self.requester.bid_value.0)?;
                                builder.set_tricks(&[self.requester.tricks_value.0])?;
                                let hand = builder.build()?;
                                Ok(hand)
                            })();

                            if hand_result.is_ok() {
                                ui.close();
                            }
                            return Some(PendingHand::Classical(hand_result));
                        }
                    }
                    None
                },
            );
            hand_res
        });
        Ok(resp)
    }

    fn show_point_modal_ui(&mut self, ui: &egui::Ui) -> ModalResponse<bool> {
        if self.requester.points.is_none() {
            self.requester.points = Some([0, 0, 0]);
        }
        egui::Modal::new("points modal".into()).show(ui.ctx(), |ui| {
            let mut points_ready = false;
            if let Err(e) = self.requester.show_points(ui) {
                error!("error: {e}");
            }
            egui::Sides::new().show(
                ui,
                |_| {},
                |ui| {
                    if ui.button("Ok").clicked() {
                        points_ready = true;
                        ui.close();
                    }
                },
            );
            points_ready
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
            if let Some(bid) = hand.bid {
                ui.label(format!("Bid: {bid}"));
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Tricks: ");
                for (id, score) in &hand.contractors_tricks {
                    let name = players[id.idx()].clone();
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
