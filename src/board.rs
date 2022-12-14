use crate::{
    consts::{TetriminoType, Delta},
    piece::Position,
};
use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct SelectedTetrimino {
    pub tetrimino_type: TetriminoType,
    pub position: Position,
}

#[derive(Bundle, Copy, Clone)]
pub struct Board {
    pub active_piece: SelectedTetrimino,
}