use spacetimedb_sdk::{credentials, Error};

use crate::module_bindings::{DbConnection, ErrorContext, Identity};

const HOST: &str = "http://localhost:3000";
const DB_NAME: &str = "tetris-mp";

pub struct NetClient {
    pub conn: DbConnection,
    pub room_code: Option<String>,
    pub is_owner: bool,
    pub start_sent: bool,
    pub tick: u64,
}

impl NetClient {
    pub fn connect() -> Result<Self, Error> {
        let conn = DbConnection::builder()
            .on_connect(on_connected)
            .on_connect_error(on_connect_error)
            .on_disconnect(on_disconnected)
            .with_token(creds_store().load().ok())
            .with_module_name(DB_NAME)
            .with_uri(HOST)
            .build()?;

        conn.subscription_builder()
            .subscribe_to_all_tables();

        Ok(Self {
            conn,
            room_code: None,
            is_owner: false,
            start_sent: false,
            tick: 0,
        })
    }
}

fn creds_store() -> credentials::File {
    credentials::File::new(DB_NAME)
}

fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
    let _ = creds_store().save(token);
}

fn on_connect_error(_ctx: &ErrorContext, _err: Error) {}

fn on_disconnected(_ctx: &ErrorContext, _err: Option<Error>) {}
