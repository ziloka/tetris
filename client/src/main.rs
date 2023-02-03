use std::cell::Cell;

use macroquad::{
    main,
    prelude::{
        is_key_pressed, is_mouse_button_down, mouse_position, KeyCode, MouseButton, BLACK, WHITE,
    },
    text::draw_text,
    time::get_fps,
    window::{clear_background, next_frame, screen_height, screen_width},
};

use tetris::{
    board::Board,
    consts::{vec2, GameState, Vec2, HEIGHT, WIDTH},
};

mod drawer;
mod game;
mod input;
mod settings;

use game::Game;

#[main("Tetris")]
async fn main() {
    let left_top_corner = Cell::new(vec2(200.0, 10.0));
    let block_size = Cell::new(30.0);
    let debug = Cell::new(false);
    let mut game = Game::new(&left_top_corner, &block_size, &debug);

    loop {
        let block_size_temp = (screen_height() / (HEIGHT * 1.25)).min(screen_width() / (WIDTH * 1.25));
        block_size.set(block_size_temp);
        left_top_corner.set(vec2(block_size_temp * 6.0, block_size_temp));

        match game.board.game_state {
            GameState::OpenSettings => {
                game.input.settings.draw_menu();
            }
            GameState::Playing => {
                clear_background(BLACK);
                modify_board_bricks(&left_top_corner, &mut game.board, &block_size);
                game.input.handle(&mut game.board);
                game.board.draw(&game.drawer);
            }
            GameState::Paused => {
                todo!();
            }
            GameState::GameOver => {
                todo!();
            }
        }
        handle_keyboard_input(&mut game.board, &debug);
        draw_text(
            format!("fps: {}", get_fps()).as_str(),
            10.0,
            20.0,
            20.0,
            WHITE,
        );
        next_frame().await;
    }
}

fn handle_keyboard_input(board: &mut Board, debug: &Cell<bool>) {
    if is_key_pressed(KeyCode::Escape) {
        match board.game_state {
            GameState::Playing => {
                board.game_state = GameState::OpenSettings;
            }
            GameState::OpenSettings => {
                board.game_state = GameState::Playing;
            }
            _ => panic!("Not implemented yet"),
        }
    } else if is_key_pressed(KeyCode::F3) {
        debug.set(!debug.get());
    }
}

fn modify_board_bricks(left_top_corner: &Cell<Vec2>, board: &mut Board, block_size: &Cell<f32>) {
    let left_top_corner = left_top_corner.get();
    let (x, y) = mouse_position();
    let x = ((x - left_top_corner.x) / block_size.get()).floor() as usize;
    let y = ((y - left_top_corner.y) / block_size.get()).floor() as usize;
    let brick = vec![vec2(0.0, 0.0)];

    if is_mouse_button_down(MouseButton::Left)
        && !board.conflict(&brick, vec2(x as f32, y as f32), false)
    {
        board.add_brick(x, y, (105, 105, 105));
    } else if is_mouse_button_down(MouseButton::Right)
        && !board.conflict(&brick, vec2(x as f32, y as f32), false)
    {
        board.remove_brick(x, y);
    }
}
