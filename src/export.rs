//! Export CSV partagé entre l'interface console et l'interface graphique.

use std::fs::write;

use anyhow::Result;
use chrono::Local;

use crate::store::Store;

/// Exporte toutes les transactions vers un fichier CSV.
/// Renvoie `None` s'il n'y a rien à exporter, sinon le chemin créé.
pub fn export_file(store: &Store) -> Result<Option<String>> {
    if store.count_transactions()? == 0 {
        return Ok(None);
    }
    let rows = store.all_transactions()?;
    let date = Local::now().naive_local().format("%Y%m%d-%H%M%S");
    let path = format!("export-mon-budget-{date}.csv");
    let mut csv = String::from("\u{FEFF}date;compte;categorie;type;montant_euros;note\n");
    for row in rows {
        let sign = if row.kind == "depense" { "-" } else { "" };
        csv.push_str(&format!(
            "{};{};{};{};{}{};{}\n",
            row.date,
            csv_field(&row.account),
            csv_field(&row.category),
            if row.kind == "revenu" { "revenu" } else { "depense" },
            sign,
            format!("{}.{:02}", row.amount_cents / 100, row.amount_cents % 100),
            csv_field(&row.note)
        ));
    }
    write(&path, csv)?;
    Ok(Some(path))
}

fn csv_field(value: &str) -> String {
    if value.contains(';') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::csv_field;

    #[test]
    fn escapes_csv_fields() {
        assert_eq!(csv_field("simple"), "simple");
        assert_eq!(csv_field("a;b"), "\"a;b\"");
        assert_eq!(csv_field("a\"b"), "\"a\"\"b\"");
        assert_eq!(csv_field("multi\nligne"), "\"multi\nligne\"");
    }
}
