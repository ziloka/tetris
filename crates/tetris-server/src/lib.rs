use spacetimedb::{reducer, table, Identity, ReducerContext, Table};
use tetris_core::logic::{board::{Board, BoardSnapshot}, score::Action};

const STATUS_LOBBY: u8 = 0;
const STATUS_ACTIVE: u8 = 1;

#[table(name = room, public)]
pub struct Room {
    #[primary_key]
    pub code: String,
    pub owner: Identity,
    pub max_players: u8,
    pub status: u8,
}

#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub room_code: String,
    pub seat: u8,
    pub is_ready: bool,
}

#[table(name = match_state, public)]
pub struct MatchState {
    #[primary_key]
    pub room_code: String,
    pub tick: u64,
    pub rng_seed: u64,
    pub status: u8,
}

#[table(name = pending_input, public)]
pub struct PendingInput {
    #[primary_key]
    pub identity: Identity,
    pub room_code: String,
    pub tick: u64,
    pub input_bits: u16,
}

#[table(name = board_state, public)]
pub struct BoardState {
    #[primary_key]
    pub identity: Identity,
    pub room_code: String,
    pub width: u8,
    pub height: u8,
    pub cells: Vec<u8>,
    pub active_tetromino: u8,
    pub active_dots: Vec<i8>,
    pub active_rotation_index: i8,
    pub active_previous_rotation_index: Option<i8>,
    pub active_previous_offset_kick: Option<u8>,
    pub hold_tetromino: Option<u8>,
    pub preview_pieces: Vec<u8>,
    pub score: u64,
    pub game_state: u8,
    pub rng_state: u64,
    pub rng_inc: u64,
    pub rng_bag: Vec<u8>,
    pub rng_index: u8,
    pub last_action: Option<u8>,
    pub combo: i32,
    pub back_to_back: bool,
    pub pending_garbage: u32,
}

#[reducer]
pub fn create_room(ctx: &ReducerContext, code: String) -> Result<(), String> {
    if ctx.db.room().iter().any(|room| room.code == code) {
        return Err("Room already exists.".to_string());
    }
    let room = Room {
        code: code.clone(),
        owner: ctx.sender,
        max_players: 2,
        status: STATUS_LOBBY,
    };
    ctx.db.room().insert(room);
    ctx.db.player().insert(Player {
        identity: ctx.sender,
        room_code: code,
        seat: 0,
        is_ready: true,
    });
    Ok(())
}

#[reducer]
pub fn join_room(ctx: &ReducerContext, code: String) -> Result<(), String> {
    let room = ctx
        .db
        .room()
        .iter()
        .find(|room| room.code == code)
        .ok_or_else(|| "Room not found.".to_string())?;
    if room.status != STATUS_LOBBY {
        return Err("Room already started.".to_string());
    }
    let players: Vec<Player> = ctx
        .db
        .player()
        .iter()
        .filter(|player| player.room_code == code)
        .collect();
    if players.len() >= room.max_players as usize {
        return Err("Room is full.".to_string());
    }
    let seat = players.len() as u8;
    ctx.db.player().insert(Player {
        identity: ctx.sender,
        room_code: code,
        seat,
        is_ready: true,
    });
    Ok(())
}

#[reducer]
pub fn leave_room(ctx: &ReducerContext) -> Result<(), String> {
    let player = ctx
        .db
        .player()
        .iter()
        .find(|player| player.identity == ctx.sender)
        .ok_or_else(|| "Player not in room.".to_string())?;
    let room_code = player.room_code.clone();
    ctx.db.player().identity().delete(&ctx.sender);
    cleanup_room(ctx, &room_code);
    Ok(())
}

#[reducer]
pub fn start_match(ctx: &ReducerContext, code: String) -> Result<(), String> {
    let room = ctx
        .db
        .room()
        .iter()
        .find(|room| room.code == code)
        .ok_or_else(|| "Room not found.".to_string())?;
    if room.owner != ctx.sender {
        return Err("Only the room owner can start the match.".to_string());
    }
    let players: Vec<Player> = ctx
        .db
        .player()
        .iter()
        .filter(|player| player.room_code == code)
        .collect();
    if players.len() < 2 {
        return Err("Need two players to start.".to_string());
    }
    ctx.db.match_state().insert(MatchState {
        room_code: code.clone(),
        tick: 0,
        rng_seed: ctx.random::<u64>(),
        status: STATUS_ACTIVE,
    });
    ctx.db.room().code().update(Room {
        status: STATUS_ACTIVE,
        ..room
    });
    for player in players {
        let seed = ctx.random::<u64>() ^ (player.seat as u64);
        let board = Board::new(seed as usize);
        let snapshot = board.to_snapshot();
        ctx.db.board_state().insert(BoardState {
            identity: player.identity,
            room_code: code.clone(),
            width: snapshot.width,
            height: snapshot.height,
            cells: snapshot.cells,
            active_tetromino: snapshot.active_tetromino,
            active_dots: snapshot.active_dots,
            active_rotation_index: snapshot.active_rotation_index,
            active_previous_rotation_index: snapshot.active_previous_rotation_index,
            active_previous_offset_kick: snapshot.active_previous_offset_kick,
            hold_tetromino: snapshot.hold_tetromino,
            preview_pieces: snapshot.preview_pieces,
            score: snapshot.score,
            game_state: snapshot.game_state,
            rng_state: snapshot.rng_state,
            rng_inc: snapshot.rng_inc,
            rng_bag: snapshot.rng_bag,
            rng_index: snapshot.rng_index,
            last_action: snapshot.last_action,
            combo: -1,
            back_to_back: false,
            pending_garbage: 0,
        });
    }
    Ok(())
}

#[reducer]
pub fn submit_input(
    ctx: &ReducerContext,
    room_code: String,
    input_bits: u16,
    client_tick: u64,
) -> Result<(), String> {
    if ctx
        .db
        .player()
        .iter()
        .all(|player| player.identity != ctx.sender)
    {
        return Err("Player not in room.".to_string());
    }
    let row = PendingInput {
        identity: ctx.sender,
        room_code,
        tick: client_tick,
        input_bits,
    };
    if ctx.db.pending_input().identity().find(&ctx.sender).is_some() {
        ctx.db.pending_input().identity().update(row);
    } else {
        ctx.db.pending_input().insert(row);
    }
    Ok(())
}

#[reducer]
pub fn tick(ctx: &ReducerContext, room_code: String) -> Result<(), String> {
    let match_state = ctx
        .db
        .match_state()
        .iter()
        .find(|state| state.room_code == room_code)
        .ok_or_else(|| "Match not found.".to_string())?;
    if match_state.status != STATUS_ACTIVE {
        return Ok(());
    }

    let players: Vec<Player> = ctx
        .db
        .player()
        .iter()
        .filter(|player| player.room_code == room_code)
        .collect();
    let identities: Vec<Identity> = players.iter().map(|player| player.identity).collect();
    for identity in identities {
        let mut board_state = match ctx.db.board_state().identity().find(&identity) {
            Some(state) => state,
            None => continue,
        };
        if board_state.game_state != 0 {
            continue;
        }
        let input_bits = ctx
            .db
            .pending_input()
            .identity()
            .find(&identity)
            .map(|input| input.input_bits)
            .unwrap_or(0);
        if input_bits != 0 {
            ctx.db.pending_input().identity().delete(&identity);
        }

        let mut board = Board::from_snapshot(&snapshot_from_state(&board_state));

        let locked_by_drop = board.apply_input_bits(input_bits);
        let locked_by_gravity = if locked_by_drop { true } else { board.tick_gravity() };

        if locked_by_drop || locked_by_gravity {
            let (garbage, combo, back_to_back) = compute_garbage(
                board.last_action.as_ref(),
                board_state.combo,
                board_state.back_to_back,
            );
            board_state.combo = combo;
            board_state.back_to_back = back_to_back;
            if garbage > 0 {
                if let Some(opponent) = opponents_for(&players, identity) {
                    if let Some(mut opponent_state) =
                        ctx.db.board_state().identity().find(&opponent)
                    {
                        opponent_state.pending_garbage =
                            opponent_state.pending_garbage.saturating_add(garbage);
                        ctx.db.board_state().identity().update(opponent_state);
                    }
                }
            }
        }

        if board_state.pending_garbage > 0 && board.last_action.is_none() {
            let hole = (ctx.random::<u8>() as usize) % board_state.width as usize;
            board.apply_garbage(board_state.pending_garbage, hole);
            board_state.pending_garbage = 0;
        }

        let snapshot = board.to_snapshot();
        apply_snapshot(&mut board_state, snapshot);
        ctx.db.board_state().identity().update(board_state);
    }

    ctx.db
        .match_state()
        .room_code()
        .update(MatchState {
            tick: match_state.tick + 1,
            ..match_state
        });

    Ok(())
}

fn cleanup_room(ctx: &ReducerContext, room_code: &str) {
    let remaining_players: Vec<Player> = ctx
        .db
        .player()
        .iter()
        .filter(|player| player.room_code == room_code)
        .collect();
    if remaining_players.is_empty() {
        if let Some(room) = ctx.db.room().iter().find(|room| room.code == room_code) {
            ctx.db.room().code().delete(&room.code);
        }
        if let Some(state) = ctx
            .db
            .match_state()
            .iter()
            .find(|state| state.room_code == room_code)
        {
            ctx.db.match_state().room_code().delete(&state.room_code);
        }
        for board in ctx
            .db
            .board_state()
            .iter()
            .filter(|board| board.room_code == room_code)
        {
            ctx.db.board_state().identity().delete(&board.identity);
        }
        for input in ctx
            .db
            .pending_input()
            .iter()
            .filter(|input| input.room_code == room_code)
        {
            ctx.db.pending_input().identity().delete(&input.identity);
        }
    }
}

fn opponents_for(players: &[Player], identity: Identity) -> Option<Identity> {
    players
        .iter()
        .find(|player| player.identity != identity)
        .map(|player| player.identity)
}

fn compute_garbage(
    action: Option<&Action>,
    combo: i32,
    back_to_back: bool,
) -> (u32, i32, bool) {
    let mut garbage = 0;
    let mut next_combo = combo;
    let mut next_b2b = back_to_back;
    let mut difficult = false;

    let base = match action {
        Some(Action::Single) => 0,
        Some(Action::Double) => 1,
        Some(Action::Triple) => 2,
        Some(Action::Tetris) => {
            difficult = true;
            4
        }
        Some(Action::TSpinMiniNoLines) => 0,
        Some(Action::TSpinMiniSingle) => {
            difficult = true;
            1
        }
        Some(Action::TSpinMiniDouble) => {
            difficult = true;
            2
        }
        Some(Action::TSpinNoLines) => {
            difficult = true;
            0
        }
        Some(Action::TSpinSingle) => {
            difficult = true;
            2
        }
        Some(Action::TSpinDouble) => {
            difficult = true;
            4
        }
        Some(Action::TSpinTriple) => {
            difficult = true;
            6
        }
        _ => 0,
    };

    if base > 0 {
        next_combo += 1;
    } else {
        next_combo = -1;
    }

    if difficult {
        if next_b2b {
            garbage += 1;
        }
        next_b2b = true;
    } else if base > 0 {
        next_b2b = false;
    }

    garbage += base as u32;
    if next_combo > 0 {
        garbage += (next_combo as u32).saturating_sub(1);
    }

    (garbage, next_combo, next_b2b)
}

fn snapshot_from_state(state: &BoardState) -> BoardSnapshot {
    BoardSnapshot {
        width: state.width,
        height: state.height,
        cells: state.cells.clone(),
        active_tetromino: state.active_tetromino,
        active_dots: state.active_dots.clone(),
        active_rotation_index: state.active_rotation_index,
        active_previous_rotation_index: state.active_previous_rotation_index,
        active_previous_offset_kick: state.active_previous_offset_kick,
        hold_tetromino: state.hold_tetromino,
        preview_pieces: state.preview_pieces.clone(),
        score: state.score,
        game_state: state.game_state,
        rng_state: state.rng_state,
        rng_inc: state.rng_inc,
        rng_bag: state.rng_bag.clone(),
        rng_index: state.rng_index,
        last_action: state.last_action,
    }
}

fn apply_snapshot(state: &mut BoardState, snapshot: BoardSnapshot) {
    state.width = snapshot.width;
    state.height = snapshot.height;
    state.cells = snapshot.cells;
    state.active_tetromino = snapshot.active_tetromino;
    state.active_dots = snapshot.active_dots;
    state.active_rotation_index = snapshot.active_rotation_index;
    state.active_previous_rotation_index = snapshot.active_previous_rotation_index;
    state.active_previous_offset_kick = snapshot.active_previous_offset_kick;
    state.hold_tetromino = snapshot.hold_tetromino;
    state.preview_pieces = snapshot.preview_pieces;
    state.score = snapshot.score;
    state.game_state = snapshot.game_state;
    state.rng_state = snapshot.rng_state;
    state.rng_inc = snapshot.rng_inc;
    state.rng_bag = snapshot.rng_bag;
    state.rng_index = snapshot.rng_index;
    state.last_action = snapshot.last_action;
}
