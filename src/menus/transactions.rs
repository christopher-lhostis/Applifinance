//! Saisie et consultation des transactions (revenus et dépenses).

use anyhow::Result;

use crate::models::{Category, TransactionRow};
use crate::store::Store;
use crate::ui;

const RECENT_LIMIT: i64 = 15;

pub fn menu(store: &mut Store) -> Result<()> {
    loop {
        println!("\n{}", ui::header("Transactions"));
        let recent = store.last_transactions(RECENT_LIMIT)?;
        show(&recent);
        println!();
        println!(" 1) Ajouter une transaction");
        println!(" 2) Modifier une transaction");
        println!(" 3) Supprimer une transaction");
        println!(" 4) Voir toutes les transactions d'un mois");
        println!(" 0) Retour");
        match ui::read_line("Votre choix :").as_str() {
            "1" => add(store)?,
            "2" => edit(store)?,
            "3" => remove(store)?,
            "4" => list_month(store)?,
            "0" | "" => return Ok(()),
            other => println!("  {}", ui::warning(&format!("Choix inconnu : {other}"))),
        }
    }
}

fn show(rows: &[TransactionRow]) {
    if rows.is_empty() {
        println!("(aucune transaction)");
        return;
    }
    for row in rows {
        let sign = if row.kind == "revenu" { "+" } else { "-" };
        println!(
            " #{}  {}  {:<18} {:<16} {sign}{}  {} ({})",
            row.id,
            row.date,
            row.category,
            row.account,
            ui::euros(row.amount_cents),
            if row.note.is_empty() { "—" } else { &row.note },
            row.kind
        );
    }
}

fn pick_from(rows: &[TransactionRow], action: &str) -> Option<TransactionRow> {
    if rows.is_empty() {
        return None;
    }
    println!("Quelle transaction {action} ? (Entrée seule pour annuler)");
    ui::pick("Numéro : ", rows.len()).map(|index| rows[index].clone())
}

fn pick_category(categories: &[Category]) -> Option<Category> {
    println!("Catégorie : (Entrée seule pour annuler)");
    for (index, category) in categories.iter().enumerate() {
        let marker = if category.is_income() { "Revenu " } else { "Dépens" };
        println!(" {:>2}. [{marker}] {}", index + 1, category.name);
    }
    ui::pick("Numéro de catégorie : ", categories.len()).map(|index| categories[index].clone())
}

fn add(store: &mut Store) -> Result<()> {
    let accounts = store.list_accounts()?;
    if accounts.is_empty() {
        println!("{}", ui::warning("⚠ Créez d'abord un compte (menu Comptes)."));
        return Ok(());
    }
    println!("Sur quel compte ? (Entrée seule pour annuler)");
    let labels: Vec<String> = accounts.iter().map(|account| account.name.clone()).collect();
    for (index, label) in labels.iter().enumerate() {
        println!(" {}. {label}", index + 1);
    }
    let Some(account_index) = ui::pick("Numéro : ", labels.len()) else {
        return Ok(());
    };
    let Some(category) = pick_category(&store.categories()?) else {
        return Ok(());
    };
    let cents = ui::read_amount_cents("Montant en € :", 0);
    if cents <= 0 {
        println!("Montant annulé.");
        return Ok(());
    }
    let date = ui::read_date("Date :");
    println!("Note (optionnelle, Entrée pour passer) :");
    let note = ui::read_line("> ");
    store.add_transaction(accounts[account_index].id, category.id, cents, &date, &note)?;
    println!(
        "{}",
        ui::success(&format!(
            "✔ {} de {} enregistré le {date} ({}).",
            if category.is_income() { "Revenu" } else { "Dépense" },
            ui::euros(cents),
            category.name
        ))
    );
    Ok(())
}

fn edit(store: &mut Store) -> Result<()> {
    let recent = store.last_transactions(RECENT_LIMIT)?;
    if recent.is_empty() {
        println!("Aucune transaction à modifier.");
        return Ok(());
    }
    let Some(row) = pick_from(&recent, "modifier") else {
        return Ok(());
    };
    println!(
        "Modification de la transaction #{} (Entrée seule = garder la valeur actuelle) :",
        row.id
    );
    println!(
        "Actuel : {} — {} — {} — {} — note : « {} »",
        row.date,
        row.category,
        row.account,
        ui::euros(row.amount_cents),
        row.note
    );

    let date = {
        let value = ui::read_line(&format!("Nouvelle date [actuelle = {}] :", row.date));
        if value.is_empty() {
            row.date.clone()
        } else if chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_ok() {
            value
        } else {
            println!("  ⚠ Date invalide, valeur actuelle conservée.");
            row.date.clone()
        }
    };
    let cents = ui::read_amount_cents(
        &format!("Nouveau montant en € [actuel = {}] :", ui::euros(row.amount_cents)),
        row.amount_cents,
    );
    println!("Nouvelle note (Entrée = garder) :");
    let note = ui::read_line("> ");
    let note = if note.is_empty() { row.note.clone() } else { note };

    let categories = store.categories()?;
    let category_id = {
        let value = ui::read_line("Nouvelle catégorie (numéro, Entrée = garder) :");
        match value.parse::<usize>() {
            Ok(n) if (1..=categories.len()).contains(&n) => categories[n - 1].id,
            _ => categories
                .iter()
                .find(|category| category.name == row.category)
                .map(|category| category.id)
                .unwrap_or(0),
        }
    };
    let accounts = store.list_accounts()?;
    let Some(account) = accounts.iter().find(|account| account.name == row.account) else {
        println!("  ⚠ Compte introuvable, modification annulée.");
        return Ok(());
    };
    if category_id == 0 {
        println!("  ⚠ Catégorie introuvable, modification annulée.");
        return Ok(());
    }
    store.update_transaction(row.id, account.id, category_id, cents, &date, &note)?;
    println!("{}", ui::success(&format!("✔ Transaction #{} mise à jour.", row.id)));
    Ok(())
}

fn remove(store: &mut Store) -> Result<()> {
    let recent = store.last_transactions(RECENT_LIMIT)?;
    if recent.is_empty() {
        println!("Aucune transaction à supprimer.");
        return Ok(());
    }
    let Some(row) = pick_from(&recent, "supprimer") else {
        return Ok(());
    };
    if ui::confirm(&format!(
        "Supprimer « {} {} {} ({}) » ?",
        row.date,
        row.category,
        ui::euros(row.amount_cents),
        row.note
    )) {
        store.delete_transaction(row.id)?;
        println!("{}", ui::success("✔ Transaction supprimée."));
    } else {
        println!("Suppression annulée.");
    }
    Ok(())
}

fn list_month(store: &mut Store) -> Result<()> {
    let month = ui::read_month("Mois à afficher :");
    let rows = store.transactions_in_month(&month)?;
    println!("\n─── Transactions de {month} ───");
    show(&rows);
    Ok(())
}
