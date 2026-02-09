use std::env;

use spacetimedb_sdk::{credentials, DbContext, Error, Identity};

use crate::module_bindings::{DbConnection, ErrorContext};

const DEFAULT_HOST: &str = "https://maincloud.spacetimedb.com";
const DEFAULT_DB: &str = "c200c68e0798a5be4e7eae538b449372e398f028ec59b85c0d9d9a44e2ee6822";

pub struct NetClient {
    pub conn: DbConnection,
    pub room_code: Option<String>,
    pub is_owner: bool,
    pub start_sent: bool,
    pub tick: u64,
}

impl NetClient {
    pub fn connect() -> Result<Self, Error> {
        let token = creds_store().load().ok().flatten();
        let host = host();
        let db = db_name();
        let conn = DbConnection::builder()
            .on_connect(on_connected)
            .on_connect_error(on_connect_error)
            .on_disconnect(on_disconnected)
            .with_token(token)
            .with_module_name(db)
            .with_uri(host)
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
    credentials::File::new(db_name())
}

fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
    let _ = creds_store().save(token);
}

fn on_connect_error(_ctx: &ErrorContext, _err: Error) {}

fn on_disconnected(_ctx: &ErrorContext, _err: Option<Error>) {}

pub fn host() -> String {
    env::var("TETRIS_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string())
}

pub fn db_name() -> String {
    env::var("TETRIS_DB").unwrap_or_else(|_| DEFAULT_DB.to_string())
}
