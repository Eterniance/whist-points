use std::error::Error;

use crate::ui::{HandBuilderGUI, hands::HandsHistoric};
use egui::vec2;
use egui_extras::{Column, TableBuilder};
use log::{debug, error};
use whist_game::{
    Players, PlayersBuilder,
    contracts::{Contract, default_contracts},
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WhistApp {
    pub players_state: PlayersState,
    pub player_field: String,
    pub contracts: Vec<Contract>,
    pub hand_builder: Option<HandBuilderGUI>,
    pub current_contract_idx: usize,
    pub pending: bool,
    pub historic: HandsHistoric,
    pub hand_detail: Option<usize>,
}

impl Default for WhistApp {
    fn default() -> Self {
        let contracts = default_contracts();
        Self {
            contracts,
            players_state: Default::default(),
            player_field: Default::default(),
            hand_builder: Default::default(),
            current_contract_idx: Default::default(),
            pending: Default::default(),
            historic: Default::default(),
            hand_detail: Default::default(),
        }
    }
}

impl WhistApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_style(egui::Style::default());
        cc.egui_ctx.set_pixels_per_point(2.);

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.button_padding = vec2(10., 8.);
        style.spacing.interact_size = vec2(60., 30.);
        style
            .text_styles
            .get_mut(&egui::TextStyle::Body)
            .expect("Default settings")
            .size = 18.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Heading)
            .expect("Default settings")
            .size = 32.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Button)
            .expect("Default settings")
            .size = 18.0;
        cc.egui_ctx.set_style(style);

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    pub fn reset_game(&mut self) {
        *self = Default::default();
    }

    pub fn select_players_ui(&mut self, ui: &mut egui::Ui) {
        let mut state = std::mem::take(&mut self.players_state);
        match &mut state {
            PlayersState::Building(players_builder) => {
                let mut should_build = false;
                ui.horizontal(|ui| {
                    ui.label("Add a new player:");
                    let response = ui.text_edit_singleline(&mut self.player_field);
                    let enter_pressed =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    let button_clicked = ui
                        .add_enabled(players_builder.players.len() < 4, egui::Button::new("Add"))
                        .on_disabled_hover_text("Already 4 players")
                        .clicked();

                    if enter_pressed || button_clicked {
                        let player_name = std::mem::take(&mut self.player_field);

                        match players_builder.add_player(&player_name) {
                            Ok(4) => {
                                should_build = true;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                error!("{e}");
                            }
                        }
                    }
                    response.request_focus();
                });

                player_grid(ui, players_builder);

                if should_build {
                    state = state.build().unwrap();
                    if let PlayersState::Playing(players) = &state {
                        self.hand_builder = Some(HandBuilderGUI::new(players.clone()));
                    }
                }
            }

            PlayersState::Playing(_) => {}
        }
        self.players_state = state;
    }

    pub fn select_gamemode_ui(&mut self, ui: &mut egui::Ui) {
        let contracts = &self.contracts;
        let current_contract_name = contracts
            .get(self.current_contract_idx)
            .expect("Index should be inbound")
            .name
            .clone();
        egui::ComboBox::from_label("Select gamemode")
            .selected_text(current_contract_name)
            .show_ui(ui, |ui| {
                for (idx, contract) in contracts.iter().enumerate() {
                    ui.selectable_value(&mut self.current_contract_idx, idx, contract.name.clone());
                }
            });
    }

    pub fn score_table_ui(&mut self, ui: &mut egui::Ui) {
        let headers_height = 20.0;
        TableBuilder::new(ui)
            .columns(
                Column::remainder()
                    .auto_size_this_frame(true)
                    .at_least(60.0),
                4,
            )
            .striped(true)
            .cell_layout(egui::Layout::top_down(egui::Align::Center))
            .sense(egui::Sense::click())
            .stick_to_bottom(true)
            .max_scroll_height(200.0)
            .header(headers_height, |mut header| {
                for name in &self
                    .players_state
                    .players()
                    .expect("Builder phase finished")
                    .names()
                {
                    header.col(|ui| {
                        ui.add(egui::Label::new(name).truncate());
                        // ui.add(egui::Separator::default().grow(5.0));
                    });
                }
            })
            .body(|mut body| {
                for (row_index, (_, scores)) in (&self.historic).into_iter().enumerate() {
                    body.row(headers_height, |mut row| {
                        for score in scores {
                            row.col(|ui| {
                                ui.label(format!("{score}"));
                            });
                        }
                        if row.response().clicked() {
                            self.hand_detail = Some(row_index);
                        }
                    });
                }
            });
    }
}

impl eframe::App for WhistApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Reset game").clicked() {
                    (*self).reset_game();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Whist Calculator");
            ui.separator();

            if self.players_state.is_building() {
                self.select_players_ui(ui);
                return;
            }

            self.score_table_ui(ui);
            ui.separator();

            self.select_gamemode_ui(ui);

            if let Some(row_idx) = self.hand_detail {
                let resp = self.historic.show_hand(
                    ui,
                    row_idx,
                    &self
                        .players_state
                        .players()
                        .expect("Builder phase finished")
                        .names(),
                );
                if resp.should_close() {
                    self.hand_detail = None;
                }
            }

            if ui.button("New hand").clicked() {
                self.pending = true;
                self.hand_builder.as_mut().unwrap().new_hand(
                    self.contracts
                        .get(self.current_contract_idx)
                        .expect("Inbound")
                        .clone(),
                );
                debug!("{}", self.current_contract_idx);
            }
            if self.pending
                && let Ok(resp) = self.hand_builder.as_mut().unwrap().ui(
                    ui,
                    self
                        .players_state
                        .players()
                        .expect("Builder phase finished"),
                )
            {
                if let Some(result) = resp.inner {
                    match result {
                        Ok(hand) => {
                            if let Ok(scores) = hand.get_scores() {
                                self.players_state
                                    .players_mut()
                                    .expect("Builder phase finished")
                                    .update_score(&scores).unwrap();
                                self.historic.push(hand.as_recap(scores));
                            } else {
                                error!("Error : Wrong Score");
                            }
                        }
                        Err(e) => error!("{e}"),
                    }
                } else if resp.should_close() {
                    self.pending = false;
                }
            }

            if ui.button("Remove last hand").clicked() {
                self.historic.remove_last();
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

fn player_grid(ui: &mut egui::Ui, players_builder: &PlayersBuilder) {
    egui::Grid::new("players_list")
        .striped(true)
        .show(ui, |ui| {
            for player in &players_builder.players {
                ui.label(format!("Player: {}", player.name));
                ui.label(format!("Score: {}", player.score));
                ui.end_row();
            }
        });
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum PlayersState {
    Building(PlayersBuilder),
    Playing(Players),
}

impl Default for PlayersState {
    fn default() -> Self {
        Self::Building(PlayersBuilder::default())
    }
}

impl PlayersState {
    fn build(self) -> Result<Self, Box<dyn Error>> {
        match self {
            Self::Building(builder) => {
                let players = builder.build()?;
                Ok(Self::Playing(players))
            }
            Self::Playing(_) => Err("Players already set".into()),
        }
    }

    fn is_building(&self) -> bool {
        matches!(self, Self::Building(_))
    }

    fn players(&self) -> Option<&Players> {
        match self {
            Self::Playing(players) => Some(players),
            Self::Building(_) => None,
        }
    }

    fn players_mut(&mut self) -> Option<&mut Players> {
        match self {
            Self::Playing(players) => Some(players),
            Self::Building(_) => None,
        }
    }
}
