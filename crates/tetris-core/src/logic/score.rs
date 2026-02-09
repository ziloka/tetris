#[derive(Clone)]
pub enum Action {
    Single,
    Double,
    Triple,
    Tetris, // line clear
    // https://tetris.wiki/T-Spin
    TSpinMiniNoLines,
    TSpinMiniSingle,
    TSpinMiniDouble,
    TSpinNoLines,
    TSpinSingle,
    TSpinDouble,
    TSpinTriple,
    BackToBackDifficultLineClears,
    Combo,
    SoftDrop,
    HardDrop,
}

impl Action {
    pub fn to_u8(&self) -> u8 {
        match *self {
            Action::Single => 0,
            Action::Double => 1,
            Action::Triple => 2,
            Action::Tetris => 3,
            Action::TSpinMiniNoLines => 4,
            Action::TSpinMiniSingle => 5,
            Action::TSpinMiniDouble => 6,
            Action::TSpinNoLines => 7,
            Action::TSpinSingle => 8,
            Action::TSpinDouble => 9,
            Action::TSpinTriple => 10,
            Action::BackToBackDifficultLineClears => 11,
            Action::Combo => 12,
            Action::SoftDrop => 13,
            Action::HardDrop => 14,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Action::Single),
            1 => Some(Action::Double),
            2 => Some(Action::Triple),
            3 => Some(Action::Tetris),
            4 => Some(Action::TSpinMiniNoLines),
            5 => Some(Action::TSpinMiniSingle),
            6 => Some(Action::TSpinMiniDouble),
            7 => Some(Action::TSpinNoLines),
            8 => Some(Action::TSpinSingle),
            9 => Some(Action::TSpinDouble),
            10 => Some(Action::TSpinTriple),
            11 => Some(Action::BackToBackDifficultLineClears),
            12 => Some(Action::Combo),
            13 => Some(Action::SoftDrop),
            14 => Some(Action::HardDrop),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match *self {
            Action::Single => "Single",
            Action::Double => "Double",
            Action::Triple => "Triple",
            Action::Tetris => "Tetris",
            Action::TSpinMiniNoLines => "T-Spin Mini No Lines",
            Action::TSpinNoLines => "T-Spin No Lines",
            Action::TSpinMiniSingle => "T-Spin Mini Single",
            Action::TSpinSingle => "T-Spin Single",
            Action::TSpinMiniDouble => "T-Spin Mini Double",
            Action::TSpinDouble => "T-Spin Double",
            Action::TSpinTriple => "T-Spin Triple",
            Action::BackToBackDifficultLineClears => "Back-to-Back Difficult Line Clears",
            Action::Combo => "Combo",
            Action::SoftDrop => "Soft Drop",
            Action::HardDrop => "Hard Drop",
        }
    }

    pub fn add_score(&self, level: &f64, action_score: &f64) -> f64 {
        match *self {
            Action::Single => {
                100. * level
                // 100 × level
            }
            Action::Double => {
                300. * level
                // 300 × level
            }
            Action::Triple => {
                500. * level
                // 500 × level
            }
            Action::Tetris => {
                800. * level
                // 800 × level; difficult
            }
            Action::TSpinMiniNoLines => {
                100. * level
                // 100 × level
            }
            Action::TSpinNoLines => {
                400. * level
                // 400 × level
            }
            Action::TSpinMiniSingle => {
                200. * level
                // 200 × level; difficult
            }
            Action::TSpinSingle => {
                800. * level
                // 800 × level; difficult
            }
            Action::TSpinMiniDouble => {
                400. * level
                // 400 × level; difficult
            }
            Action::TSpinDouble => {
                1200. * level
                // 1200 × level; difficult
            }
            Action::TSpinTriple => {
                1600. * level
                // 1600 × level; difficult
            }
            Action::BackToBackDifficultLineClears => {
                // Action score × 1.5 (excluding soft drop and hard drop)
                action_score * 1.5
            }
            Action::Combo => {
                // 50 × combo count × level
                26.
            }
            Action::SoftDrop => {
                // 1 per cell
                1.
            }
            Action::HardDrop => {
                // 2 per cell
                2.
            }
        }
    }
}
