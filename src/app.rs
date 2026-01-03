use log::{debug, error};
use std::sync::Arc;
use whist::{
    game::{
        players::Players,
        rules::{Contract, GameRules, select_rules},
    },
    gamemodes::Score as _,
};

use crate::whist::{HandBuilderGUI, hands::HandsHistoric};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct WhistApp {
    pub players: Players,
    pub player_field: String,
    pub gamerules: Option<(GameRules, Vec<Arc<Contract>>)>,
    pub hand_builder: HandBuilderGUI,
    pub current_contract_idx: usize,
    pub pending: bool,
    pub historic: HandsHistoric,
}

impl WhistApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.hand_builder.players = app.players.clone();
            app
        } else {
            Default::default()
        }
    }

    pub fn reset_game(&mut self) {
        *self = Default::default();
    }

    pub fn select_rules_ui(&mut self, ui: &mut egui::Ui) {
        let mut selected = None;
        ui.label("Select gamemode:");
        ui.selectable_value(&mut selected, Some(GameRules::Dutch), "Dutch")
            .on_hover_text("Basic game mode");
        ui.selectable_value(&mut selected, Some(GameRules::French), "French")
            .on_hover_text("Some other rules");

        if let Some(rules) = selected {
            let modes = select_rules(&rules).into_iter().map(Arc::new).collect();
            self.gamerules = Some((rules, modes));
        }
    }

    pub fn select_players_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Add a new palyer:");
            let response = ui.text_edit_singleline(&mut self.player_field);
            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            let button_clicked = ui
                .add_enabled(self.players.list.len() < 4, egui::Button::new("Add"))
                .on_disabled_hover_text("Already 4 players")
                .clicked();

            if enter_pressed || button_clicked {
                let player_name = self.player_field.clone();
                self.player_field.clear();

                match self.players.add_player(player_name) {
                    Ok(4) => {
                        self.hand_builder = HandBuilderGUI::new(self.players.clone());
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }

            response.request_focus();
        });

        player_grid(ui, &self.players);
    }

    pub fn select_gamemode_ui(&mut self, ui: &mut egui::Ui) {
        let contracts = &self.gamerules.as_ref().expect("Checked if set").1;
        let current_contract_name = contracts
            .get(self.current_contract_idx)
            .expect("Index should be inbound")
            .gamemode
            .name()
            .clone();
        egui::ComboBox::from_label("Select gamemode")
            .selected_text(current_contract_name)
            .show_ui(ui, |ui| {
                for (idx, contract) in contracts.iter().enumerate() {
                    ui.selectable_value(
                        &mut self.current_contract_idx,
                        idx,
                        contract.gamemode.name(),
                    );
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
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);

                if ui.button("Reset").clicked() {
                    (*self).reset_game();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Whist Calculator");

            if self.gamerules.is_none() {
                self.select_rules_ui(ui);
                return;
            }

            ui.label(format!(
                "Current rules: {}",
                self.gamerules
                    .as_ref()
                    .expect("Value set earlier")
                    .0
                    .clone()
            ));

            if self.players.list.len() != 4 {
                self.select_players_ui(ui);
                return;
            }

            player_grid(ui, &self.players);

            self.select_gamemode_ui(ui);

            if ui.button("new_hand").clicked() {
                self.pending = true;
                self.hand_builder.new_hand(Arc::clone(
                    self.gamerules
                        .as_ref()
                        .expect("Checked if set")
                        .1
                        .get(self.current_contract_idx)
                        .expect("Inbound"),
                ));
                debug!("{}", self.current_contract_idx);
            }
            if self.pending {
                if let Ok(resp) = self.hand_builder.ui(ui, &self.players) {
                    if let Some(result) = resp.inner {
                        debug!("Some result found");
                        match result {
                            Ok(hand) => {
                                let score = hand.get_score();
                                self.players.update_score(&hand.contractors, score);
                                debug!("score : {score}");
                            }
                            Err(e) => error!("{e}"),
                        }
                    } else if resp.should_close() {
                        debug!("No result");
                        self.pending = false;
                    }
                }
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

fn player_grid(ui: &mut egui::Ui, players: &Players) {
    egui::Grid::new("players_list")
        .striped(true)
        .show(ui, |ui| {
            for player in &players.list {
                ui.label(format!("Player: {}", player.name));
                ui.label(format!("Score: {}", player.score));
                ui.end_row();
            }
        });
}
