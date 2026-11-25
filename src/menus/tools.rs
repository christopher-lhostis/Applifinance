//! Outils console : export CSV.

use anyhow::Result;

use crate::export::export_file;
use crate::store::Store;

pub fn export_csv(store: &Store) -> Result<()> {
    match export_file(store)? {
        Some(path) => println!("✔ Export réussi : {path}"),
        None => println!("Aucune transaction à exporter."),
    }
    Ok(())
}
