use std::ops::{Add, Div, Mul, Sub};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<Vec2> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<f32> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Sub<f32> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

pub const GRAY: (u8, u8, u8) = (128, 128, 128);
pub const CUSTOM_GARBAGE: (u8, u8, u8) = (105, 105, 105);

pub const TETROMINO_TYPES: [Tetromino; 7] = [
    Tetromino::I,
    Tetromino::J,
    Tetromino::L,
    Tetromino::O,
    Tetromino::S,
    Tetromino::T,
    Tetromino::Z,
];

// https://en.wikipedia.org/wiki/Tetromino
#[derive(Default, Clone, Copy, Debug)]
pub enum Tetromino {
    #[default]
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl Tetromino {
    pub fn to_u8(&self) -> u8 {
        match *self {
            Tetromino::I => 0,
            Tetromino::J => 1,
            Tetromino::L => 2,
            Tetromino::O => 3,
            Tetromino::S => 4,
            Tetromino::T => 5,
            Tetromino::Z => 6,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Tetromino::I),
            1 => Some(Tetromino::J),
            2 => Some(Tetromino::L),
            3 => Some(Tetromino::O),
            4 => Some(Tetromino::S),
            5 => Some(Tetromino::T),
            6 => Some(Tetromino::Z),
            _ => None,
        }
    }

    // RGB value of color
    pub fn get_color(&self) -> (u8, u8, u8) {
        match *self {
            Tetromino::I => (0, 255, 255),
            Tetromino::J => (0, 0, 255),
            Tetromino::L => (255, 129, 0),
            Tetromino::O => (255, 255, 0),
            Tetromino::S => (0, 255, 0),
            Tetromino::T => (255, 0, 255),
            Tetromino::Z => (255, 0, 0),
        }
    }

    pub fn get_structure(&self) -> [Vec2; 4] {
        match *self {
            Tetromino::I => [
                vec2(0.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(2.0, 0.0),
                vec2(1.0, 0.0),
            ],
            Tetromino::J => [
                vec2(0.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, -1.0),
                vec2(1.0, 0.0),
            ],
            Tetromino::L => [
                vec2(0.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(1.0, -1.0),
                vec2(1.0, 0.0),
            ],
            Tetromino::O => [
                vec2(0.0, 0.0),
                vec2(1.0, -1.0),
                vec2(0.0, -1.0),
                vec2(1.0, 0.0),
            ],
            Tetromino::S => [
                vec2(0.0, 0.0),
                vec2(0.0, -1.0),
                vec2(-1.0, 0.0),
                vec2(1.0, -1.0),
            ],
            Tetromino::T => [
                vec2(0.0, 0.0),
                vec2(0.0, -1.0),
                vec2(-1.0, 0.0),
                vec2(1.0, 0.0),
            ],
            Tetromino::Z => [
                vec2(0.0, 0.0),
                vec2(-1.0, -1.0),
                vec2(0.0, -1.0),
                vec2(1.0, 0.0),
            ],
        }
    }

    // figure out what index to recieve from the offset table
    // for 90 degree turns
    pub fn find_offset_row_90(first: i8, second: i8) -> usize {
        if (first, second) == (0, 1) {
            // 0->R
            0
        } else if (first, second) == (1, 0) {
            // R->0
            1
        } else if (first, second) == (1, 2) {
            // R->2
            2
        } else if (first, second) == (2, 1) {
            // 2->R
            3
        } else if (first, second) == (2, 3) {
            // 2->L
            4
        } else if (first, second) == (3, 2) {
            // L->2
            5
        } else if (first, second) == (3, 0) {
            // L->0
            6
        } else if (first, second) == (0, 3) {
            // 0->L
            7
        } else {
            0
        }
    }

    pub fn find_offset_row_180(first: i8, second: i8) -> usize {
        if (first, second) == (0, 2) {
            0
        } else if (first, second) == (2, 0) {
            1
        } else if (first, second) == (1, 3) {
            2
        } else if (first, second) == (3, 1) {
            3
        } else {
            0
        }
    }

    // https://harddrop.com/wiki/SRS
    // # four possible rotation states: 0, R, 2, L (Spawn, clockwise, 180, counterclockwise)
    // this returns rotation offset data for all pieces and the active piece will use
    // find_offset_row() to figure out which row is the offset that it needs
    pub fn get_offset_data(&self) -> Vec<Vec<Vec2>> {
        match *self {
            Tetromino::O => vec![
                vec![
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(1.0, 0.0),
                    vec2(1.0, -1.0),
                    vec2(0.0, -2.0),
                    vec2(1.0, -2.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(-1.0, 0.0),
                    vec2(-1.0, -1.0),
                    vec2(0.0, -2.0),
                    vec2(-1.0, -2.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 0.0),
                ],
            ],
            Tetromino::I => vec![
                vec![
                    vec2(0.0, 0.0),
                    vec2(-2.0, 0.0),
                    vec2(1.0, 0.0),
                    vec2(-2.0, -1.0),
                    vec2(1.0, 2.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(2.0, 0.0),
                    vec2(-1.0, 0.0),
                    vec2(2.0, 1.0),
                    vec2(-1.0, -2.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(-1.0, 0.0),
                    vec2(2.0, 0.0),
                    vec2(-1.0, 2.0),
                    vec2(2.0, -1.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(1.0, 0.0),
                    vec2(-2.0, 0.0),
                    vec2(1.0, -2.0),
                    vec2(-2.0, 1.0),
                ],
                vec![
                    vec2(0.0, 0.0),
                    vec2(0.0, 1.0),
                    vec2(0.0, 2.0),
                    vec2(1.0, 1.0),
                    vec2(1.0, 2.0),
                    vec2(0.0, -1.0),
                    vec2(0.0, -2.0),
                    vec2(1.0, -1.0),
                    vec2(1.0, -2.0),
                    vec2(-1.0, 0.0),
                    vec2(0.0, 3.0),
                    vec2(0.0, -3.0),
                ],
            ],
            _ => vec![
                vec![
                    vec2(-1.0, 0.0),
                    vec2(-2.0, 0.0),
                    vec2(1.0, 0.0),
                    vec2(2.0, 0.0),
                    vec2(0.0, 1.0),
                ],
                vec![
                    vec2(0.0, 1.0),
                    vec2(0.0, 2.0),
                    vec2(0.0, -1.0),
                    vec2(0.0, -2.0),
                    vec2(-1.0, 0.0),
                ],
                vec![
                    vec2(1.0, 0.0),
                    vec2(2.0, 0.0),
                    vec2(-1.0, 0.0),
                    vec2(-2.0, 0.0),
                    vec2(0.0, -1.0),
                ],
                vec![
                    vec2(0.0, 1.0),
                    vec2(0.0, 2.0),
                    vec2(0.0, -1.0),
                    vec2(0.0, -2.0),
                    vec2(1.0, 0.0),
                ],
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Piece {
    pub tetromino: Tetromino,
    pub dots: Vec<Vec2>,
    pub rotation_index: i8,
    pub previous_rotation_index: Option<i8>,
    pub previous_offset_kick: Option<usize>, // there are only 5 kicks
}

#[derive(Clone)]
pub enum State {
    Playing,
    GameOver,
}

impl State {
    pub fn to_u8(&self) -> u8 {
        match *self {
            State::Playing => 0,
            State::GameOver => 1,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(State::Playing),
            1 => Some(State::GameOver),
            _ => None,
        }
    }
}
