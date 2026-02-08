use std::cell::Cell;

use macroquad::{
    input::{get_char_pressed, is_key_pressed},
    main,
    prelude::{draw_text, is_key_down, KeyCode, BLACK, WHITE},
    window::{clear_background, next_frame, screen_height, screen_width},
};

use tetris::{
    core::logic::{
        board::{
            Board, BoardSnapshot, INPUT_HARD_DROP, INPUT_HOLD, INPUT_LEFT, INPUT_RIGHT,
            INPUT_ROTATE_180, INPUT_ROTATE_CCW, INPUT_ROTATE_CW, INPUT_SOFT_DROP,
        },
        consts::vec2,
    },
    drawer::Drawer,
    net::NetClient,
};

use tetris::module_bindings::BoardState;

enum UiMode {
    Menu,
    Lobby,
    Playing,
}

#[main("Tetris Multiplayer")]
async fn main() {
    let mut net = match NetClient::connect() {
        Ok(client) => client,
        Err(_) => return,
    };

    let mut mode = UiMode::Menu;
    let mut input_code = String::new();
    let mut create_mode = true;

    let board_offset = Cell::new(vec2(40.0, screen_height() - 40.0));
    let opp_offset = Cell::new(vec2(420.0, screen_height() - 40.0));
    let block_size = Cell::new(20.0);
    let debug = Cell::new(false);
    let drawer = Drawer {
        bottom_left_corner: &board_offset,
        block_size: &block_size,
        debug: &debug,
    };
    let opponent_drawer = Drawer {
        bottom_left_corner: &opp_offset,
        block_size: &block_size,
        debug: &debug,
    };

    loop {
        clear_background(BLACK);
        let _ = net.conn.frame_tick();

        match mode {
            UiMode::Menu => {
                draw_text("Multiplayer Tetris", 40.0, 40.0, 30.0, WHITE);
                draw_text(
                    "Press C to create, J to join. Type room code, Enter to confirm.",
                    40.0,
                    70.0,
                    20.0,
                    WHITE,
                );
                if is_key_pressed(KeyCode::C) {
                    create_mode = true;
                }
                if is_key_pressed(KeyCode::J) {
                    create_mode = false;
                }
                if let Some(ch) = get_char_pressed() {
                    if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                        input_code.push(ch.to_ascii_uppercase());
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    input_code.pop();
                }
                draw_text(
                    &format!(
                        "Mode: {} | Code: {}",
                        if create_mode { "Create" } else { "Join" },
                        input_code
                    ),
                    40.0,
                    100.0,
                    22.0,
                    WHITE,
                );

                if is_key_pressed(KeyCode::Enter) && !input_code.is_empty() {
                    if create_mode {
                        let _ = net.conn.reducers.create_room(input_code.clone());
                        net.is_owner = true;
                    } else {
                        let _ = net.conn.reducers.join_room(input_code.clone());
                    }
                    net.room_code = Some(input_code.clone());
                    mode = UiMode::Lobby;
                }
            }
            UiMode::Lobby => {
                draw_text("Waiting for opponent...", 40.0, 40.0, 26.0, WHITE);
                if let Some(room_code) = net.room_code.clone() {
                    let player_count = net
                        .conn
                        .db
                        .player()
                        .iter()
                        .filter(|player| player.room_code == room_code)
                        .count();
                    draw_text(
                        &format!("Room: {} | Players: {}", room_code, player_count),
                        40.0,
                        70.0,
                        20.0,
                        WHITE,
                    );
                    if net.is_owner && player_count >= 2 && !net.start_sent {
                        let _ = net.conn.reducers.start_match(room_code.clone());
                        net.start_sent = true;
                    }
                    if net.conn.db.match_state().iter().any(|state| {
                        state.room_code == room_code && state.status == 1
                    }) {
                        mode = UiMode::Playing;
                    }
                }
            }
            UiMode::Playing => {
                if let Some(room_code) = net.room_code.clone() {
                    let input_bits = collect_input_bits();
                    if input_bits != 0 {
                        let _ = net
                            .conn
                            .reducers
                            .submit_input(room_code.clone(), input_bits, net.tick);
                    }
                    let _ = net.conn.reducers.tick(room_code.clone());
                    net.tick += 1;
                }

                let boards: Vec<BoardState> = net.conn.db.board_state().iter().collect();
                let (my_board, opp_board) = split_boards(&boards);

                if let Some(board) = my_board {
                    draw_board_snapshot(&drawer, board);
                }
                if let Some(board) = opp_board {
                    draw_board_snapshot(&opponent_drawer, board);
                }
            }
        }

        next_frame().await;
    }
}

fn collect_input_bits() -> u16 {
    let mut bits = 0;
    if is_key_down(KeyCode::Left) {
        bits |= INPUT_LEFT;
    }
    if is_key_down(KeyCode::Right) {
        bits |= INPUT_RIGHT;
    }
    if is_key_down(KeyCode::Down) {
        bits |= INPUT_SOFT_DROP;
    }
    if is_key_pressed(KeyCode::Space) {
        bits |= INPUT_HARD_DROP;
    }
    if is_key_pressed(KeyCode::Z) {
        bits |= INPUT_ROTATE_CCW;
    }
    if is_key_pressed(KeyCode::X) {
        bits |= INPUT_ROTATE_CW;
    }
    if is_key_pressed(KeyCode::A) {
        bits |= INPUT_ROTATE_180;
    }
    if is_key_pressed(KeyCode::LeftShift) {
        bits |= INPUT_HOLD;
    }
    bits
}

fn split_boards(boards: &[BoardState]) -> (Option<&BoardState>, Option<&BoardState>) {
    let mut iter = boards.iter();
    (iter.next(), iter.next())
}

fn draw_board_snapshot(drawer: &Drawer<'_>, board_state: &BoardState) {
    let snapshot = BoardSnapshot {
        width: board_state.width,
        height: board_state.height,
        cells: board_state.cells.clone(),
        active_tetromino: board_state.active_tetromino,
        active_dots: board_state.active_dots.clone(),
        active_rotation_index: board_state.active_rotation_index,
        active_previous_rotation_index: board_state.active_previous_rotation_index,
        active_previous_offset_kick: board_state.active_previous_offset_kick,
        hold_tetromino: board_state.hold_tetromino,
        preview_pieces: board_state.preview_pieces.clone(),
        score: board_state.score,
        game_state: board_state.game_state,
        rng_state: board_state.rng_state,
        rng_inc: board_state.rng_inc,
        rng_bag: board_state.rng_bag.clone(),
        rng_index: board_state.rng_index,
        last_action: board_state.last_action,
    };

    let board = Board::from_snapshot(&snapshot);
    drawer.draw_tetrominos(&board.positions);
    drawer.draw_ghost_piece(&board);
    drawer.draw_current_tetromino(&board.active_piece);
}
