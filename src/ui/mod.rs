pub mod hands;
pub mod requester;
use std::error::Error;

use egui::emath::Numeric;
pub use hands::HandBuilderGUI;
use thiserror::Error;
use whist_game::Tricks;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("An error has occur: {0}")]
    ImpossibleState(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TricksGui(Tricks);

impl TricksGui {
    pub fn new(num: u8) -> Result<Self, Box<dyn Error>> {
        let tricks = Tricks::new(num)?;
        Ok(Self(tricks))
    }
}

impl Numeric for TricksGui {
    const INTEGRAL: bool = true;
    const MAX: Self = Self(Tricks::MAX_TRICKS);
    const MIN: Self = Self(Tricks::MIN_TRICKS);

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self.0.get() as f64
    }

    #[inline(always)]
    fn from_f64(num: f64) -> Self {
        let tricks = Tricks::new(num as u8).unwrap_or(Tricks::MIN_TRICKS);
        Self(tricks)
    }
}
