//! Budgets mensuels par catégorie, avec suivi visuel et alertes.

use anyhow::Result;

use crate::store::Store;
use crate::ui;

pub fn menu(store: &mut Store) -> Result<()> {
    let mut month = ui::current_month();
    loop {
        println!("\n{}", ui::paint_bold(&ui::MISTRAL_PALETTE[2], &format!("─── Budgets du mois {month} ───")));
        let budgets = store.budgets_for_month(&month)?;
        if budgets.is_empty() {
            println!("(aucun budget défini pour ce mois)");
        } else {
            for line in &budgets {
                let ratio = if line.limit_cents > 0 {
                    line.spent_cents as f64 / line.limit_cents as f64
                } else {
                    0.0
                };
                let percent = (ratio * 100.0).round() as i64;
                let color = if ratio > 1.0 {
                    &ui::MISTRAL_PALETTE[4]
                } else if ratio >= 0.9 {
                    &ui::MISTRAL_PALETTE[1]
                } else {
                    &ui::MISTRAL_PALETTE[2]
                };
                let alert_text = if ratio > 1.0 {
                    format!("  {}", ui::alert("⚠ DÉPASSÉ !"))
                } else if ratio >= 0.9 {
                    format!("  {}", ui::warning("⚠ Attention, proche de la limite"))
                } else {
                    String::new()
                };
                println!(
                    " {:<18} {} {} / {} ({percent} %){alert_text}",
                    ui::paint_normal(&ui::MISTRAL_PALETTE[3], &line.category_name),
                    ui::paint_normal(color, &ui::progress_bar(ratio, 20)),
                    ui::euros(line.spent_cents),
                    ui::euros(line.limit_cents)
                );
            }
        }
        println!();
        println!(" 1) Changer de mois");
        println!(" 2) Définir / modifier un budget");
        println!(" 3) Supprimer un budget");
        println!(" 0) Retour");
        match ui::read_line("Votre choix :").as_str() {
            "1" => month = ui::read_month("Mois :"),
            "2" => set(store, &month)?,
            "3" => remove(store, &month)?,
            "0" | "" => return Ok(()),
            other => println!("  {}", ui::warning(&format!("Choix inconnu : {other}"))),
        }
    }
}

fn set(store: &mut Store, month: &str) -> Result<()> {
    let categories = store.categories()?;
    let expense_categories: Vec<_> = categories
        .iter()
        .filter(|category| !category.is_income())
        .collect();
    println!("Pour quelle catégorie budgétiser ? (Entrée seule pour annuler)");
    for (index, category) in expense_categories.iter().enumerate() {
        println!(" {:>2}. {}", index + 1, category.name);
    }
    let Some(index) = ui::pick("Numéro de catégorie : ", expense_categories.len()) else {
        return Ok(());
    };
    let category = expense_categories[index];
    let limit = ui::read_amount_cents(
        &format!("Limite mensuelle en € pour « {} » :", category.name),
        0,
    );
    if limit <= 0 {
        println!("Budget annulé.");
        return Ok(());
    }
    store.set_budget(category.id, month, limit)?;
    println!(
        "{}",
        ui::success(&format!(
            "✔ Budget de {} fixé pour « {} » en {month}.",
            ui::euros(limit),
            category.name
        ))
    );
    Ok(())
}

fn remove(store: &mut Store, month: &str) -> Result<()> {
    let budgets = store.budgets_for_month(month)?;
    if budgets.is_empty() {
        println!("Aucun budget à supprimer ce mois-ci.");
        return Ok(());
    }
    println!("Quel budget supprimer ? (Entrée seule pour annuler)");
    for (index, line) in budgets.iter().enumerate() {
        println!(
            " {}. {} — {}",
            index + 1,
            line.category_name,
            ui::euros(line.limit_cents)
        );
    }
    if let Some(index) = ui::pick("Numéro : ", budgets.len()) {
        let line = &budgets[index];
        if ui::confirm(&format!(
            "Supprimer le budget « {} » de {month} ?",
            line.category_name
        )) {
            store.delete_budget(line.category_id, month)?;
            println!("{}", ui::success("✔ Budget supprimé."));
        }
    }
    Ok(())
}
