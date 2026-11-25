use anyhow::Result;

use mon_budget::{menus, store::Store};

fn main() -> Result<()> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "mon-budget.db".to_string());
    let mut store = Store::open(&db_path)?;
    menus::main_menu(&mut store)
}
