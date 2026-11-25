//! Menus de l'application.

mod accounts;
mod budgets;
mod reports;
mod tools;
mod transactions;

use anyhow::Result;

use crate::store::Store;
use crate::ui;

pub fn main_menu(store: &mut Store) -> Result<()> {
    println!("\nBienvenue dans Mon Budget !");
    println!("Toutes les données sont stockées localement, dans un fichier à côté du programme.");
    loop {
        println!("\n{}", ui::app_banner());
        let total = store.total_balance()?;
        let total_color = if total < 0 { &ui::MISTRAL_PALETTE[4] } else { &ui::MISTRAL_PALETTE[2] };
        println!(
            "Solde total : {}",
            ui::paint_bold(total_color, &ui::euros(total))
        );
        if store.count_accounts()? == 0 {
            println!("(Commencez par créer votre premier compte : option 1)");
        }
        println!();
        println!(" 1) Comptes");
        println!(" 2) Transactions");
        println!(" 3) Budgets mensuels");
        println!(" 4) Rapport mensuel");
        println!(" 5) Exporter en CSV");
        println!(" 0) Quitter");
        match ui::read_line("Votre choix :").as_str() {
            "1" => accounts::menu(store)?,
            "2" => transactions::menu(store)?,
            "3" => budgets::menu(store)?,
            "4" => reports::menu(store)?,
            "5" => tools::export_csv(store)?,
            "0" | "q" | "Q" => {
                println!("{}", ui::success("À bientôt !"));
                return Ok(());
            }
            other => {
                if other.is_empty() {
                    return Ok(());
                }
                println!("  {}", ui::warning(&format!("Choix inconnu : {other}")));
            }
        }
    }
}
