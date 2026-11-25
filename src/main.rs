//! Programme principal : interface web locale dans le navigateur.

use anyhow::{bail, Result};

use mon_budget::store::Store;
use mon_budget::web;

const FIRST_PORT: u16 = 8630;
const PORTS_TO_TRY: u16 = 10;

fn main() -> Result<()> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "mon-budget.db".to_string());
    let store = Store::open(&db_path)?;
    for port in FIRST_PORT..FIRST_PORT + PORTS_TO_TRY {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return web::serve(store, port);
        }
    }
    bail!("Impossible de trouver un port libre (essayé {FIRST_PORT} à {}).", FIRST_PORT + PORTS_TO_TRY - 1)
}
