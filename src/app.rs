use egui::Button;
use log::{error, info};
use whist::game::players::{Contractors, PlayerId, Players};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct WhistApp {
    players: Players,
    player_field: String,
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
            let Self {
                players,
                player_field,
                pending,
            } = self;

            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Whist Calculator");

            ui.horizontal(|ui| {
                ui.label("Add a new palyer:");
                let response = ui.text_edit_singleline(player_field);
                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                let button_clicked = ui
                    .add_enabled(players.list.len() < 4, egui::Button::new("Add"))
                    .on_disabled_hover_text("Already 4 players")
                    .clicked();

                if enter_pressed || button_clicked {
                    let player_name = player_field.clone();
                    player_field.clear();

                    if let Err(e) = players.add_player(player_name) {
                        error!("{e}");
                    }
                }

                response.request_focus();
            });

            egui::Grid::new("players_list")
                .striped(true)
                .show(ui, |ui| {
                    for player in &players.list {
                        ui.label(format!("Player: {}", player.name));
                        ui.label(format!("Score: {}", player.score));
                        ui.end_row();
                    }
                });

            if ui.add(egui::Button::new("select player")).clicked() {
                *pending = true;
            }
            if *pending {
                let modal = names_modal(ui, &players.names());
                if modal.should_close() {
                    *pending = false;
                }
            }

            if ui.button("add_score").clicked() {
                players.update_score(&Contractors::Team(PlayerId::new(0), PlayerId::new(1)), 2);
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
