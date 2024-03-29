use std::cell::Cell;

use macroquad::{
    prelude::{draw_rectangle, Color, BLACK, GREEN, ORANGE, RED, VIOLET, WHITE},
    shapes::{draw_circle_lines, draw_rectangle_lines},
    text::draw_text,
    time::{get_fps, get_time},
};

use crate::core::logic::{
    board::{Board, POSITIONS},
    consts::{vec2, Piece, Tetromino, Vec2, GRAY},
};

pub struct Drawer<'a> {
    pub bottom_left_corner: &'a Cell<Vec2>,
    pub block_size: &'a Cell<f32>,
    pub debug: &'a Cell<bool>,
}

impl<'a> Drawer<'a> {
    pub fn draw_current_tetromino(&self, active_piece: &Piece) {
        let block_size = self.block_size.get();
        let bottom_left_corner = self.bottom_left_corner.get();
        let debug = self.debug.get();
        let (r, g, b) = active_piece.tetromino.get_color();
        let origin = active_piece.dots[0];

        if debug {
            self.draw_debug_info();

            // for debugging purposes (ensure pieces are in the true SRS rotations)
            // https://harddrop.com/wiki/File:SRS-true-rotations.png
            let (min, max) = match active_piece.tetromino {
                Tetromino::I => (-2, 2),
                _ => (-1, 1),
            };
            // draw black rectangle behind tetromino
            for i in min..=max {
                for j in min..=max {
                    let x = bottom_left_corner.x + (origin.x + i as f32) * block_size;
                    let y = bottom_left_corner.y - (origin.y + j as f32) * block_size - block_size;
                    draw_rectangle(x, y, block_size, block_size, BLACK);
                }
            }
        }

        // draw current block
        active_piece.dots.iter().for_each(|position| {
            let x = bottom_left_corner.x + position.x * block_size;
            let y = bottom_left_corner.y - position.y * block_size - block_size;
            draw_rectangle(x, y, block_size, block_size, Color::from_rgba(r, g, b, 255));

            draw_rectangle_lines(x, y, block_size, block_size, 2.0, BLACK);
        });

        if debug {
            // draw circle on origin on active_piece (continued debug purposes)
            draw_circle_lines(
                bottom_left_corner.x + origin.x * block_size + block_size / 2.0,
                bottom_left_corner.y - origin.y * block_size + block_size / 2.0 - block_size,
                block_size / 2.0,
                2.0,
                WHITE,
            );

            // debugging for detecting tspins
            if matches!(active_piece.tetromino, Tetromino::T) {
                let fl = vec2(origin.x - 1.0, origin.y + 1.0) * block_size;
                let fr = vec2(origin.x + 1.0, origin.y + 1.0) * block_size;
                let bl = vec2(origin.x + 1.0, origin.y - 1.0) * block_size;
                let br = vec2(origin.x - 1.0, origin.y - 1.0) * block_size;

                let (front_left, front_right, bottom_left, bottom_right) =
                    match active_piece.rotation_index {
                        0 => (fl, fr, bl, br),
                        1 => (fr, br, fl, bl),
                        2 => (br, bl, fr, fl),
                        _ => {
                            assert!(active_piece.rotation_index == 3);
                            (bl, fl, br, fr)
                        }
                    };

                draw_rectangle_lines(
                    bottom_left_corner.x + front_left.x,
                    bottom_left_corner.y - front_left.y - block_size,
                    block_size,
                    block_size,
                    5.0,
                    RED,
                );
                draw_rectangle_lines(
                    bottom_left_corner.x + front_right.x,
                    bottom_left_corner.y - front_right.y - block_size,
                    block_size,
                    block_size,
                    5.0,
                    ORANGE,
                );
                draw_rectangle_lines(
                    bottom_left_corner.x + bottom_left.x,
                    bottom_left_corner.y - bottom_left.y - block_size,
                    block_size,
                    block_size,
                    5.0,
                    GREEN,
                );
                draw_rectangle_lines(
                    bottom_left_corner.x + bottom_right.x,
                    bottom_left_corner.y - bottom_right.y - block_size,
                    block_size,
                    block_size,
                    5.0,
                    VIOLET,
                );
            }
        }
    }

    // draw ghost piece, ghost piece appears where the harddrop would be
    pub fn draw_ghost_piece(&self, board: &Board) {
        let block_size = self.block_size.get();
        let bottom_left_corner = self.bottom_left_corner.get();

        let mut y_offset = 0.0;
        for y in 1..board.positions.len() as i32 {
            let y = y as f32 * -1.0;
            if board.conflict(&board.active_piece.dots, vec2(0.0, y), true) {
                y_offset = y + 1.0;
                break;
            }
        }

        board.active_piece.dots.iter().for_each(|position| {
            let x = bottom_left_corner.x + position.x * block_size;
            let y = bottom_left_corner.y - (position.y + y_offset) * block_size - block_size;
            draw_rectangle(x, y, block_size, block_size, WHITE);
            draw_rectangle_lines(x, y, block_size, block_size, 2.0, BLACK);
        });
    }

    pub fn draw_tetrominos(&self, positions: &POSITIONS) {
        let block_size = self.block_size.get();
        let bottom_left_corner = self.bottom_left_corner.get();
        let debug = self.debug.get();

        draw_rectangle(
            bottom_left_corner.x,
            bottom_left_corner.y - positions.len() as f32 * block_size,
            positions[0].len() as f32 * block_size,
            positions.len() as f32 * block_size,
            Color::from_rgba(GRAY.0, GRAY.1, GRAY.2, 255),
        );

        // draw all the blocks that are already on the board
        for (y, row) in positions.iter().enumerate() {
            for (x, color) in row.iter().enumerate() {
                let x = bottom_left_corner.x + x as f32 * block_size;
                let y = bottom_left_corner.y - (y as f32 + 1.0) * block_size;
                if let Some(color) = color {
                    let (r, g, b) = *color;
                    draw_rectangle(x, y, block_size, block_size, Color::from_rgba(r, g, b, 255));
                }
                draw_rectangle_lines(x, y, block_size, block_size, 2.0, BLACK);
            }
        }

        if debug {
            // draw the indexs on the side of the board
            for i in 0..(positions.len() as u32) {
                draw_text(
                    i.to_string().as_str(),
                    bottom_left_corner.x - block_size,
                    (bottom_left_corner.y - i as f32 * block_size) - block_size / 2.0,
                    20.0,
                    WHITE,
                );
            }

            for i in 0..(positions[0].len() as u32) {
                draw_text(
                    i.to_string().as_str(),
                    bottom_left_corner.x + i as f32 * block_size,
                    bottom_left_corner.y + block_size / 2.0,
                    20.0,
                    WHITE,
                );
            }
        }
    }

    pub fn draw_preview_pieces(&self, board: &Board) {
        let block_size = self.block_size.get();
        let bottom_left_corner = self.bottom_left_corner.get();
        let preview_pieces = board.preview_pieces;

        preview_pieces[0..5]
            .iter()
            .enumerate()
            .for_each(|(i, tetromino)| {
                tetromino.get_structure().iter().for_each(|position| {
                    let x = bottom_left_corner.x
                        + (board.positions[0].len() as f32 + 2.0 + position.x) * block_size;
                    let y = (bottom_left_corner.y - board.positions.len() as f32 * block_size)
                        + (i as f32 * 3.0 + position.y) * block_size;
                    let (r, g, b) = tetromino.get_color();
                    draw_rectangle(x, y, block_size, block_size, Color::from_rgba(r, g, b, 255));

                    draw_rectangle_lines(x, y, block_size, block_size, 2.0, BLACK);
                });
            });
    }

    pub fn draw_hold_piece(&self, board: &Board) {
        let block_size = self.block_size.get();
        let bottom_left_corner = self.bottom_left_corner.get();
        let debug = self.debug.get();
        let hold_piece = &board.hold_piece;

        if let Some(piece) = hold_piece {
            let dots = piece.tetromino.get_structure();
            let origin = dots[0];
            dots.iter().for_each(|position| {
                let x =
                    bottom_left_corner.x - (origin.x - position.x * block_size) + block_size * -3.0;
                let y =
                    (bottom_left_corner.y - origin.y - board.positions.len() as f32 * block_size)
                        + position.y * block_size
                        + block_size;
                let (r, g, b) = piece.tetromino.get_color();
                draw_rectangle(x, y, block_size, block_size, Color::from_rgba(r, g, b, 255));

                draw_rectangle_lines(x, y, block_size, block_size, 2.0, BLACK);
            });

            if debug {
                // draw origin dot on hold piece location
                draw_circle_lines(
                    bottom_left_corner.x - origin.x * block_size
                        + block_size * -3.0
                        + (block_size / 2.0),
                    (bottom_left_corner.y - origin.y - board.positions.len() as f32 * block_size)
                        - (block_size / 2.0 - block_size * 2.0),
                    block_size / 2.0,
                    2.0,
                    WHITE,
                );
            }
        }
    }

    pub fn draw_action_text(&self, board: &Board, text: &str) {
        let block_size = self.block_size.get();
        let bottom_left_corner = self.bottom_left_corner.get();

        draw_text(
            text,
            bottom_left_corner.x - 5.0 * block_size,
            bottom_left_corner.y - (board.positions.len() as f32 / 2.0) * block_size,
            20.0,
            WHITE,
        );
    }

    pub fn draw_debug_info(&self) {
        draw_text(
            format!("fps: {}", get_fps()).as_str(),
            10.0,
            20.0,
            20.0,
            WHITE,
        );
        draw_text(
            format!(
                "It has been {:.2} minutes since program started",
                get_time() / 60.0
            )
            .as_str(),
            10.,
            40.,
            20.,
            WHITE,
        );
    }
}
