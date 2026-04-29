#![warn(clippy::all, rust_2018_idioms)]
use whist_game::{CollectedTricks, Score, Tricks};

mod app;
pub use app::WhistApp;
mod ui;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Queens;

#[typetag::serde]
impl Score for Queens {
    fn min_tricks(&self) -> Tricks {
        Tricks::new(0).expect("Within values")
    }

    fn calculate_score(&self, tricks: CollectedTricks) -> i16 {
        match tricks.absolute.get() {
            4 => 21,
            3 => -15,
            2 => -10,
            1 => -5,
            _ => 0,
        }
    }
}
