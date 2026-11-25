//! Gestion des comptes.

use anyhow::Result;

use crate::store::Store;
use crate::ui;

pub fn menu(store: &mut Store) -> Result<()> {
    loop {
        println!("\n{}", ui::header("Comptes"));
        let accounts = store.accounts_with_balances()?;
        if accounts.is_empty() {
            println!("(aucun compte pour le moment)");
        } else {
            for (index, (account, balance)) in accounts.iter().enumerate() {
                println!(" {}. {} [{}] — {}", index + 1, account.name, account.kind, ui::euros(*balance));
            }
        }
        println!();
        println!(" 1) Ajouter un compte");
        println!(" 2) Supprimer un compte");
        println!(" 0) Retour");
        match ui::read_line("Votre choix :").as_str() {
            "1" => add(store)?,
            "2" => remove(store)?,
            "0" | "" => return Ok(()),
            other => println!("  ⚠ Choix inconnu : {other}"),
        }
    }
}

fn add(store: &mut Store) -> Result<()> {
    let name = ui::read_non_empty("Nom du compte (ex. Compte courant) :");
    let existing: Vec<String> = store
        .list_accounts()?
        .into_iter()
        .map(|account| account.name)
        .collect();
    if existing.iter().any(|existing_name| existing_name.eq_ignore_ascii_case(&name)) {
        println!("  {}", ui::warning("Un compte portant ce nom existe déjà."));
        return Ok(());
    }
    println!("Type de compte :");
    println!(" 1) Courant   2) Épargne   3) Espèces   4) Autre");
    let kind = match ui::read_line("Votre choix :").as_str() {
        "2" => "Épargne",
        "3" => "Espèces",
        "4" => "Autre",
        _ => "Courant",
    };
    let initial = ui::read_amount_cents("Solde de départ en € (Entrée = 0) :", 0);
    store.add_account(&name, kind, initial)?;
    println!(
        "{}",
        ui::success(&format!(
            "✔ Compte « {name} » créé avec un solde de départ de {}.",
            ui::euros(initial)
        ))
    );
    Ok(())
}

fn remove(store: &mut Store) -> Result<()> {
    let accounts = store.accounts_with_balances()?;
    if accounts.is_empty() {
        println!("Aucun compte à supprimer.");
        return Ok(());
    }
    println!("Quel compte supprimer ? (Entrée seule pour annuler)");
    let labels: Vec<String> = accounts
        .iter()
        .map(|(account, balance)| format!("{} [{}] — {}", account.name, account.kind, ui::euros(*balance)))
        .collect();
    for (index, label) in labels.iter().enumerate() {
        println!(" {}. {label}", index + 1);
    }
    if let Some(choice) = ui::pick("Numéro : ", labels.len()) {
        let (account, _) = &accounts[choice];
        let account = account.clone();
        let transactions = store.count_transactions_for_account(account.id)?;
        let deletion_warning = if transactions > 0 {
            format!(
                " {}",
                ui::alert(&format!(
                    "⚠ {transactions} transaction(s) liée(s) seront aussi supprimées !"
                ))
            )
        } else {
            String::new()
        };
        if ui::confirm(&format!(
            "Supprimer « {} » définitivement ?{deletion_warning}",
            account.name
        )) {
            store.delete_account(account.id)?;
            println!("{}", ui::success("✔ Compte supprimé."));
        } else {
            println!("Suppression annulée.");
        }
    }
    Ok(())
}
