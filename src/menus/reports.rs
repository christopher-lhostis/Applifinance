//! Rapport mensuel : revenus, dépenses, épargne, répartition par catégorie.

use anyhow::Result;

use crate::store::Store;
use crate::ui;

fn report_banner(month: &str) -> String {
    let line = "═".repeat(37);
    let top = ui::paint_normal(&ui::MISTRAL_PALETTE[0], &format!("╔{line}╗"));
    let middle = ui::paint_bold(
        &ui::MISTRAL_PALETTE[2],
        &format!("║        Rapport de {month}           ║"),
    );
    let bottom = ui::paint_normal(&ui::MISTRAL_PALETTE[4], &format!("╚{line}╝"));
    format!("{top}\n{middle}\n{bottom}")
}

pub fn menu(store: &mut Store) -> Result<()> {
    let month = ui::read_month("Mois du rapport :");
    let summary = store.month_summary(&month)?;
    println!("\n{}", report_banner(&month));
    println!("Revenus            : {}", ui::paint_normal(&ui::MISTRAL_PALETTE[1], &ui::euros(summary.income_cents)));
    println!("Dépenses           : {}", ui::paint_normal(&ui::MISTRAL_PALETTE[3], &ui::euros(summary.expense_cents)));
    let net = summary.income_cents - summary.expense_cents;
    let net_color = if net < 0 { &ui::MISTRAL_PALETTE[4] } else { &ui::MISTRAL_PALETTE[2] };
    println!("Reste (épargne)    : {}", ui::paint_bold(net_color, &ui::euros(net)));
    if summary.income_cents > 0 {
        let saved_ratio = net as f64 / summary.income_cents as f64;
        println!("Taux d'épargne     : {:.1} %", saved_ratio * 100.0);
    }
    if summary.by_category.is_empty() {
        println!("\nAucune dépense enregistrée ce mois-ci.");
    } else {
        println!("\nDépenses par catégorie :");
        for (name, cents) in &summary.by_category {
            let share = if summary.expense_cents > 0 {
                (*cents as f64 / summary.expense_cents as f64 * 100.0).round() as i64
            } else {
                0
            };
            println!(
                " {:<18} {}  {share} %",
                name,
                ui::euros(*cents)
            );
        }
    }
    Ok(())
}
