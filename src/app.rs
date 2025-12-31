use egui::Button;
use log::{debug, error, info};
use std::sync::Arc;
use whist::game::{
    players::{Contractors, PlayerId, Players},
    rules::{Contract, GameRules, select_rules},
};

use crate::whist::HandBuilderGUI;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct WhistApp {
    players: Arc<Players>,
    player_field: String,
    gamerules: Option<(GameRules, Vec<Arc<Contract>>)>,
    hand_builder: HandBuilderGUI,
    pending: bool,
}

impl WhistApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

                if let Some(players) = Arc::get_mut(&mut self.players) {
                    match players.add_player(player_name) {
                        Ok(4) => {
                            self.hand_builder = HandBuilderGUI::new(Arc::clone(&self.players));
                        }
                        Ok(_) => {}
                        Err(e) => {
                            error!("{e}");
                        }
                    }
                } else {
                    debug!("Players not avaible");
                }
            }

            response.request_focus();
        });

        player_grid(ui, &self.players);
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

            let Self {
                players,
                player_field,
                pending,
                gamerules,
                hand_builder,
            } = self;

            player_grid(ui, players);

            egui::ComboBox::from_label("Select gamemode").show_ui(ui, |ui| {
                for contract in &gamerules.as_ref().expect("Checked if set").1 {}
            });

            if ui.button("new_hand").clicked() {
                *pending = true;
            }
            if *pending {
                hand_builder.new_hand(Arc::clone(
                    gamerules
                        .as_ref()
                        .expect("Checked if set")
                        .1
                        .first()
                        .unwrap(),
                ));

                if let Ok(resp) = hand_builder.ui(ui) {
                    if resp.should_close() {
                        *pending = false;
                    }
                }
            }

            // if ui.add(egui::Button::new("select player")).clicked() {
            //     *pending = true;
            // }
            // if *pending {
            //     let modal = names_modal(ui, &players.names());
            //     if modal.should_close() {
            //         *pending = false;
            //     }
            // }

            // if ui.button("add_score").clicked() {
            //     Arc::get_mut(players)
            //         .expect("should be avaiblale")
            //         .update_score(&Contractors::Team(PlayerId::new(0), PlayerId::new(1)), 2);
            // }

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

fn names_modal(ui: &egui::Ui, names: &Vec<String>) -> egui::ModalResponse<()> {
    egui::Modal::new("names_display".into()).show(ui.ctx(), |ui| {
        for name in names {
            if ui.add(Button::selectable(true, name)).clicked() {
                info!("{name} clicked");
            }
        }

        egui::Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                if ui.button("Ok").clicked() {
                    ui.close();
                }
            },
        );
    })
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
