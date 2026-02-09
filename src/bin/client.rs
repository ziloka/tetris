use std::cell::Cell;

use macroquad::{
    input::{get_char_pressed, is_key_pressed, is_mouse_button_pressed, mouse_position},
    main,
    prelude::{draw_rectangle, draw_rectangle_lines, draw_text, is_key_down, Color, KeyCode, Rect, MouseButton, BLACK, WHITE},
    window::{clear_background, next_frame, screen_height, screen_width},
};

use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread;
use tetris::{
    core::logic::{
        board::{
            Board, BoardSnapshot, INPUT_HARD_DROP, INPUT_HOLD, INPUT_LEFT, INPUT_RIGHT,
            INPUT_ROTATE_180, INPUT_ROTATE_CCW, INPUT_ROTATE_CW, INPUT_SOFT_DROP,
        },
        consts::vec2,
    },
    drawer::Drawer,
    net::{db_name, host, NetClient},
};

use tetris::module_bindings::{
    create_room, join_room, start_match, submit_input, tick, BoardState,
    BoardStateTableAccess, MatchStateTableAccess, PlayerTableAccess, RoomTableAccess,
};
use spacetimedb_sdk::{table::Table, DbContext, Status};

enum UiMode {
    Menu,
    Lobby,
    Playing,
}

enum PendingAction {
    Create(String),
    Join(String),
}

#[main("Tetris Multiplayer")]
async fn main() {
    let (net_tx, net_rx) = mpsc::channel();
    let (ui_tx, ui_rx) = mpsc::channel();
    spawn_connect(net_tx.clone());
    let mut net: Option<NetClient> = None;
    let mut net_error: Option<String> = None;
    let mut callbacks_registered = false;
    let mut pending_action: Option<PendingAction> = None;

    let mut mode = UiMode::Menu;
    let mut input_code = String::new();
    let mut input_focused = true;
    let mut create_mode = true;
    let mut ui_message: Option<String> = None;

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
        if let Some(client) = net.as_ref() {
            let _ = client.conn.frame_tick();
        }

        match mode {
            UiMode::Menu => {
                draw_text("Multiplayer Tetris", 40.0, 40.0, 30.0, WHITE);
                draw_text("Create or join a room", 40.0, 70.0, 22.0, WHITE);
                match net.as_ref() {
                    Some(_) => {
                        draw_text("Connected.", 40.0, 95.0, 18.0, WHITE);
                    }
                    None => {
                        let status = net_error
                            .as_deref()
                            .unwrap_or("Connecting to server...");
                        draw_text(status, 40.0, 95.0, 18.0, WHITE);
                        draw_text(&format!("Host: {} | DB: {}", host(), db_name()), 40.0, 115.0, 18.0, WHITE);
                    }
                }
                let create_clicked = draw_button(40.0, 140.0, 140.0, 36.0, "Create", create_mode, true);
                let join_clicked = draw_button(190.0, 140.0, 140.0, 36.0, "Join", !create_mode, true);
                if create_clicked || is_key_pressed(KeyCode::C) {
                    create_mode = true;
                }
                if join_clicked || is_key_pressed(KeyCode::J) {
                    create_mode = false;
                }

                let retry_enabled = net_error.is_some() || net.is_none();
                if draw_button(340.0, 140.0, 140.0, 36.0, "Retry", false, retry_enabled)
                    || (net_error.is_some() && is_key_pressed(KeyCode::R))
                {
                    net_error = None;
                    net = None;
                    pending_action = None;
                    spawn_connect(net_tx.clone());
                }

                let input_box_clicked = draw_input_box(
                    40.0,
                    200.0,
                    300.0,
                    40.0,
                    "Room code",
                    &input_code,
                    input_focused,
                );
                if input_box_clicked {
                    input_focused = true;
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    input_focused = false;
                }

                if input_focused {
                    if let Some(ch) = get_char_pressed() {
                        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                            input_code.push(ch.to_ascii_uppercase());
                        }
                    }
                    if is_key_pressed(KeyCode::Backspace) {
                        input_code.pop();
                    }
                }

                draw_text(
                    &format!("Mode: {}", if create_mode { "Create" } else { "Join" }),
                    40.0,
                    265.0,
                    20.0,
                    WHITE,
                );

                let can_submit = net.is_some() && !input_code.is_empty();
                let submit_clicked = draw_button(
                    40.0,
                    290.0,
                    160.0,
                    40.0,
                    if create_mode { "Create Room" } else { "Join Room" },
                    false,
                    can_submit,
                );
                if (submit_clicked || is_key_pressed(KeyCode::Enter)) && !input_code.is_empty() {
                    let Some(client) = net.as_mut() else {
                        ui_message = Some("Not connected yet.".to_string());
                        next_frame().await;
                        continue;
                    };
                    if create_mode {
                        if let Err(err) = client.conn.reducers.create_room(input_code.clone()) {
                            ui_message = Some(format!("Create failed: {err}"));
                            next_frame().await;
                            continue;
                        }
                        client.is_owner = true;
                        pending_action = Some(PendingAction::Create(input_code.clone()));
                        ui_message = Some("Creating room...".to_string());
                    } else {
                        if let Err(err) = client.conn.reducers.join_room(input_code.clone()) {
                            ui_message = Some(format!("Join failed: {err}"));
                            next_frame().await;
                            continue;
                        }
                        pending_action = Some(PendingAction::Join(input_code.clone()));
                        ui_message = Some("Joining room...".to_string());
                    }
                }

                if let Some(message) = ui_message.as_deref() {
                    draw_text(message, 40.0, 345.0, 18.0, WHITE);
                }
            }
            UiMode::Lobby => {
                draw_text("Waiting room", 40.0, 40.0, 26.0, WHITE);
                if let Some(client) = net.as_mut() {
                    if let Some(room_code) = client.room_code.clone() {
                        let player_count = client
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

                        let identity = client.conn.try_identity();
                        let room = client
                            .conn
                            .db
                            .room()
                            .iter()
                            .find(|room| room.code == room_code);
                        let is_owner = room
                            .as_ref()
                            .and_then(|room| identity.map(|id| room.owner == id))
                            .unwrap_or(false);

                        if is_owner {
                            let can_start = player_count >= 2;
                            if draw_button(
                                40.0,
                                110.0,
                                160.0,
                                40.0,
                                "Start Game",
                                false,
                                can_start,
                            ) && can_start
                            {
                                let _ = client.conn.reducers.start_match(room_code.clone());
                                client.start_sent = true;
                            }
                        } else {
                            draw_text("Waiting for host to start...", 40.0, 135.0, 18.0, WHITE);
                        }

                        if client.conn.db.match_state().iter().any(|state| {
                            state.room_code == room_code && state.status == 1
                        }) {
                            mode = UiMode::Playing;
                        }
                    }
                }
            }
            UiMode::Playing => {
                if let Some(client) = net.as_mut() {
                    if let Some(room_code) = client.room_code.clone() {
                        let input_bits = collect_input_bits();
                        if input_bits != 0 {
                            let _ = client
                                .conn
                                .reducers
                                .submit_input(room_code.clone(), input_bits, client.tick);
                        }
                        let _ = client.conn.reducers.tick(room_code.clone());
                        client.tick += 1;
                    }

                    let boards: Vec<BoardState> = client.conn.db.board_state().iter().collect();
                    let (my_board, opp_board) = split_boards(&boards);

                    if let Some(board) = my_board {
                        draw_board_snapshot(&drawer, board);
                    }
                    if let Some(board) = opp_board {
                        draw_board_snapshot(&opponent_drawer, board);
                    }
                }
            }
        }

        if net.is_none() {
            match net_rx.try_recv() {
                Ok(Ok(client)) => net = Some(client),
                Ok(Err(err)) => net_error = Some(err.to_string()),
                Err(TryRecvError::Disconnected) => {
                    net_error = Some("Failed to start network client.".to_string())
                }
                Err(TryRecvError::Empty) => {}
            }
        }

        if let Ok(message) = ui_rx.try_recv() {
            ui_message = Some(message);
            pending_action = None;
        }

        if let Some(client) = net.as_ref() {
            if !callbacks_registered {
                let tx = ui_tx.clone();
                client.conn.reducers.on_create_room(move |ctx, _code| {
                    if let Status::Failed(err) = &ctx.event.status {
                        let _ = tx.send(format!("Create failed: {err}"));
                    }
                });
                let tx = ui_tx.clone();
                client.conn.reducers.on_join_room(move |ctx, _code| {
                    if let Status::Failed(err) = &ctx.event.status {
                        let _ = tx.send(format!("Join failed: {err}"));
                    }
                });
                callbacks_registered = true;
            }
        }

        if let (Some(client), Some(action)) = (net.as_mut(), pending_action.as_ref()) {
            let Some(identity) = client.conn.try_identity() else {
                next_frame().await;
                continue;
            };
            let room_code = match action {
                PendingAction::Create(code) => code,
                PendingAction::Join(code) => code,
            };
            let room_exists = client
                .conn
                .db
                .room()
                .iter()
                .any(|room| room.code == *room_code);
            let player_exists = client
                .conn
                .db
                .player()
                .iter()
                .any(|player| player.room_code == *room_code && player.identity == identity);
            if room_exists && player_exists {
                client.room_code = Some(room_code.clone());
                mode = UiMode::Lobby;
                pending_action = None;
                ui_message = None;
            }
        }

        next_frame().await;
    }
}

fn spawn_connect(sender: Sender<Result<NetClient, spacetimedb_sdk::Error>>) {
    thread::spawn(move || {
        let _ = sender.send(NetClient::connect());
    });
}

fn draw_button(x: f32, y: f32, w: f32, h: f32, label: &str, active: bool, enabled: bool) -> bool {
    let (mx, my) = mouse_position();
    let rect = Rect::new(x, y, w, h);
    let hovered = mx >= rect.x && mx <= rect.x + rect.w && my >= rect.y && my <= rect.y + rect.h;
    let clicked = enabled && hovered && is_mouse_button_pressed(MouseButton::Left);

    let bg = if !enabled {
        Color::from_rgba(60, 60, 60, 255)
    } else if active {
        Color::from_rgba(70, 110, 200, 255)
    } else if hovered {
        Color::from_rgba(90, 90, 90, 255)
    } else {
        Color::from_rgba(70, 70, 70, 255)
    };

    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 2.0, WHITE);
    draw_text(label, x + 10.0, y + h * 0.65, 20.0, WHITE);

    clicked
}

fn draw_input_box(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    placeholder: &str,
    value: &str,
    focused: bool,
) -> bool {
    let (mx, my) = mouse_position();
    let rect = Rect::new(x, y, w, h);
    let hovered = mx >= rect.x && mx <= rect.x + rect.w && my >= rect.y && my <= rect.y + rect.h;
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let border = if focused {
        Color::from_rgba(120, 160, 255, 255)
    } else {
        Color::from_rgba(160, 160, 160, 255)
    };

    draw_rectangle(x, y, w, h, Color::from_rgba(30, 30, 30, 255));
    draw_rectangle_lines(x, y, w, h, 2.0, border);

    if value.is_empty() && !focused {
        draw_text(placeholder, x + 10.0, y + h * 0.65, 18.0, Color::from_rgba(140, 140, 140, 255));
    } else {
        draw_text(value, x + 10.0, y + h * 0.65, 20.0, WHITE);
    }

    clicked
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
