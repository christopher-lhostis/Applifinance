//! Assistants de saisie et d'affichage dans le terminal.

use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::OnceLock;

use chrono::Local;

/// Palette Mistral (du jaune au rouge), en composantes RGB 24 bits.
pub const MISTRAL_PALETTE: [&str; 5] = [
    "255;216;0", "255;175;0", "255;130;5", "250;80;15", "225;5;0",
];

const RESET: &str = "\u{1b}[0m";

fn colors_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::io::stdout().is_terminal())
}

fn paint(rgb: &str, bold: bool, text: &str) -> String {
    if colors_enabled() {
        let weight = if bold { "1;" } else { "" };
        format!("\u{1b}[{weight}38;2;{rgb}m{text}{RESET}")
    } else {
        text.to_string()
    }
}

pub fn paint_normal(rgb: &str, text: &str) -> String {
    paint(rgb, false, text)
}

pub fn paint_bold(rgb: &str, text: &str) -> String {
    paint(rgb, true, text)
}

/// Applique le dégradé Mistral caractère par caractère.
pub fn gradient(text: &str) -> String {
    if !colors_enabled() {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let last = MISTRAL_PALETTE.len() - 1;
    let mut out = String::new();
    for (index, ch) in chars.iter().enumerate() {
        let step = if chars.len() <= 1 {
            0
        } else {
            index * last / (chars.len() - 1)
        };
        out.push_str(&format!("\u{1b}[38;2;{}m{ch}", MISTRAL_PALETTE[step]));
    }
    out.push_str(RESET);
    out
}

/// Titre de section : « ─── Texte ─── » en orange gras.
pub fn header(text: &str) -> String {
    paint_bold(&MISTRAL_PALETTE[2], &format!("─── {text} ───"))
}

/// Message de succès (✔) en orange Mistral.
pub fn success(text: &str) -> String {
    paint_bold(&MISTRAL_PALETTE[2], text)
}

/// Message d'avertissement (⚠) en ambre Mistral.
pub fn warning(text: &str) -> String {
    paint_bold(&MISTRAL_PALETTE[1], text)
}

/// Alerte critique (⚠) en rouge Mistral.
pub fn alert(text: &str) -> String {
    paint_bold(&MISTRAL_PALETTE[4], text)
}

/// Bandeau principal avec le dégradé de couleurs Mistral.
pub fn app_banner() -> String {
    let line = "═".repeat(34);
    let top = paint_normal(&MISTRAL_PALETTE[0], &format!("╔{line}╗"));
    let middle = format!("║{}{}{}║", " ".repeat(12), gradient("MON BUDGET"), " ".repeat(12));
    let bottom = paint_normal(&MISTRAL_PALETTE[4], &format!("╚{line}╝"));
    format!("{top}\n{middle}\n{bottom}")
}

/// Mois en cours au format AAAA-MM.
pub fn current_month() -> String {
    Local::now().naive_local().format("%Y-%m").to_string()
}

/// Affiche une invite puis lit une ligne au clavier.
/// Renvoie une chaîne vide en fin d'entrée (Ctrl+D) pour sortir proprement.
pub fn read_line(prompt: &str) -> String {
    print!("{prompt} ");
    io::stdout().flush().expect("affichage impossible");
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .unwrap_or_default();
    line.trim().to_string()
}

pub fn read_non_empty(prompt: &str) -> String {
    loop {
        let value = read_line(prompt);
        if !value.is_empty() {
            return value;
        }
        println!("  {}", warning("Veuillez saisir une valeur (Entrée seule pour annuler)."));
    }
}

/// Saisit un montant en euros (virgule ou point acceptés) et le convertit en centimes.
/// Une saisie vide renvoie `default_cents`.
pub fn read_amount_cents(prompt: &str, default_cents: i64) -> i64 {
    loop {
        let raw = read_line(prompt);
        if raw.is_empty() {
            return default_cents;
        }
        match raw.replace(',', ".").parse::<f64>() {
            Ok(value) if value > 0.0 => return (value * 100.0).round() as i64,
            _ => println!("  {}", warning("Montant invalide, exemple : 12,50")),
        }
    }
}

/// Saisit une date AAAA-MM-JJ ; vide = aujourd'hui.
pub fn read_date(prompt: &str) -> String {
    let today = Local::now().naive_local().format("%Y-%m-%d").to_string();
    loop {
        let raw = read_line(&format!("{prompt} [aujourd'hui = {today}]"));
        let value = if raw.is_empty() { today.clone() } else { raw };
        if chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_ok() {
            return value;
        }
        println!("  {}", warning("Date invalide, format attendu AAAA-MM-JJ"));
    }
}

/// Saisit un mois AAAA-MM ; vide = mois en cours.
pub fn read_month(prompt: &str) -> String {
    let current = Local::now().naive_local().format("%Y-%m").to_string();
    loop {
        let raw = read_line(&format!("{prompt} [actuel = {current}]"));
        let value = if raw.is_empty() { current.clone() } else { raw };
        if chrono::NaiveDate::parse_from_str(&format!("{value}-01"), "%Y-%m-%d").is_ok() {
            return value;
        }
        println!("  {}", warning("Mois invalide, format attendu AAAA-MM"));
    }
}

/// Demande de choisir un élément numéroté ; Entrée seule renvoie `None` (annulation).
pub fn pick(prompt: &str, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    loop {
        let raw = read_line(prompt);
        if raw.is_empty() {
            return None;
        }
        match raw.parse::<usize>() {
            Ok(n) if (1..=count).contains(&n) => return Some(n - 1),
            _ => println!("  {}", warning(&format!("Entrez un nombre entre 1 et {count}"))),
        }
    }
}

pub fn confirm(prompt: &str) -> bool {
    let answer = read_line(&format!("{prompt} (o/n)")).to_lowercase();
    answer == "o" || answer == "oui" || answer == "y" || answer == "yes"
}

/// Formate des centimes en euros, à la française : 1 234,56 €
pub fn euros(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let cents = cents.unsigned_abs();
    let units = cents / 100;
    let decimals = cents % 100;
    let digits = units.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(' ');
        }
        grouped.push(digit);
    }
    format!("{sign}{grouped},{decimals:02} €")
}

/// Barre de progression texte pour les budgets.
pub fn progress_bar(ratio: f64, width: usize) -> String {
    let clamped = ratio.clamp(0.0, 1.0);
    let filled = (clamped * width as f64).round() as usize;
    let mut bar = String::from("[");
    for index in 0..width {
        bar.push(if index < filled { '#' } else { '.' });
    }
    bar.push(']');
    bar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_euros_correctly() {
        assert_eq!(euros(0), "0,00 €");
        assert_eq!(euros(4530), "45,30 €");
        assert_eq!(euros(123456), "1 234,56 €");
        assert_eq!(euros(-123456), "-1 234,56 €");
        assert_eq!(euros(1234567), "12 345,67 €");
    }

    #[test]
    fn builds_progress_bar() {
        assert_eq!(progress_bar(0.0, 4), "[....]");
        assert_eq!(progress_bar(0.5, 4), "[##..]");
        assert_eq!(progress_bar(1.0, 4), "[####]");
        assert_eq!(progress_bar(5.0, 4), "[####]");
    }
}
