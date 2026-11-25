//! Interface graphique (eframe/egui) aux couleurs Mistral.

use std::time::{Duration, Instant};

use chrono::{Local, Months, NaiveDate};
use eframe::egui;
use egui::{Color32, ComboBox, RichText, ScrollArea, Vec2};

use crate::export::export_file;
use crate::models::{Account, BudgetLine, Category, MonthSummary, TransactionRow};
use crate::store::Store;
use crate::ui::euros;

const YEL: Color32 = Color32::from_rgb(255, 216, 0);
const AMB: Color32 = Color32::from_rgb(255, 175, 0);
const ORA: Color32 = Color32::from_rgb(255, 130, 5);
const REDO: Color32 = Color32::from_rgb(250, 80, 15);
const RED: Color32 = Color32::from_rgb(225, 5, 0);
const BACKGROUND: Color32 = Color32::from_rgb(21, 21, 29);
const PANEL: Color32 = Color32::from_rgb(29, 29, 39);
const CARD: Color32 = Color32::from_rgb(41, 41, 54);
const TEXT: Color32 = Color32::from_rgb(235, 235, 240);
const TEXT_DIM: Color32 = Color32::from_rgb(150, 150, 165);
const TEXT_DARK: Color32 = Color32::from_rgb(35, 25, 10);

#[derive(PartialEq, Clone, Copy)]
enum Page {
    Dashboard,
    Transactions,
    Budgets,
    Report,
    Export,
}

struct Toast {
    message: String,
    color: Color32,
    until: Instant,
}

#[derive(Default)]
struct TxForm {
    account_index: usize,
    category_index: usize,
    amount: String,
    date: String,
    note: String,
    error: Option<String>,
}

#[derive(Default)]
struct AccountForm {
    name: String,
    kind_index: usize,
    initial: String,
    error: Option<String>,
}

#[derive(Default)]
struct BudgetForm {
    category_index: usize,
    limit: String,
    error: Option<String>,
}

enum Dialog {
    NewTransaction(TxForm),
    EditTransaction(i64, TxForm),
    NewAccount(AccountForm),
    NewBudget(BudgetForm),
}

/// Application graphique principale.
pub struct FinanceApp {
    store: Store,
    page: Page,
    month: String,
    accounts: Vec<(Account, i64)>,
    total_cents: i64,
    categories: Vec<Category>,
    month_txs: Vec<TransactionRow>,
    recent_txs: Vec<TransactionRow>,
    budgets: Vec<BudgetLine>,
    summary: MonthSummary,
    dialog: Option<Dialog>,
    confirm_delete: Option<i64>,
    toasts: Vec<Toast>,
}

fn shift_month(month: &str, delta: i64) -> String {
    let date = NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    let shifted = if delta >= 0 {
        date.checked_add_months(Months::new(delta as u32))
    } else {
        date.checked_sub_months(Months::new((-delta) as u32))
    };
    shifted
        .unwrap_or(date)
        .format("%Y-%m")
        .to_string()
}

fn parse_amount_cents(raw: &str) -> Option<i64> {
    let value: f64 = raw.trim().replace(',', ".").parse().ok()?;
    (value > 0.0).then(|| (value * 100.0).round() as i64)
}

fn valid_date(raw: &str) -> bool {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").is_ok()
}

/// Valide le formulaire et renvoie le montant en centimes, ou renseigne l'erreur.
fn validate_tx(form: &mut TxForm) -> Option<i64> {
    if let Some(cents) = parse_amount_cents(&form.amount) {
        if valid_date(form.date.trim()) {
            return Some(cents);
        }
        form.error = Some("Date invalide, format attendu AAAA-MM-JJ".into());
    } else {
        form.error = Some("Montant invalide, exemple : 12,50".into());
    }
    None
}

fn amount_input_text(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

impl FinanceApp {
    pub fn new() -> Result<Self, String> {
        let store = Store::open("mon-budget.db").map_err(|error| error.to_string())?;
        Ok(Self {
            store,
            page: Page::Dashboard,
            month: crate::ui::current_month(),
            accounts: Vec::new(),
            total_cents: 0,
            categories: Vec::new(),
            month_txs: Vec::new(),
            recent_txs: Vec::new(),
            budgets: Vec::new(),
            summary: MonthSummary {
                income_cents: 0,
                expense_cents: 0,
                by_category: Vec::new(),
            },
            dialog: None,
            confirm_delete: None,
            toasts: Vec::new(),
        })
    }

    fn setup_theme(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = PANEL;
        style.visuals.window_fill = CARD;
        style.visuals.extreme_bg_color = BACKGROUND;
        style.visuals.faint_bg_color = CARD;
        style.visuals.widgets.noninteractive.bg_fill = CARD;
        style.visuals.widgets.inactive.bg_fill = CARD;
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 52, 68);
        ctx.set_style(style);
    }

    fn refresh_data(&mut self) {
        self.accounts = self.store.accounts_with_balances().unwrap_or_default();
        self.total_cents = self.store.total_balance().unwrap_or(0);
        self.categories = self.store.categories().unwrap_or_default();
        self.month_txs = self.store.transactions_in_month(&self.month).unwrap_or_default();
        self.recent_txs = self.store.last_transactions(10).unwrap_or_default();
        self.budgets = self.store.budgets_for_month(&self.month).unwrap_or_default();
        self.summary = self.store.month_summary(&self.month).unwrap_or(MonthSummary {
            income_cents: 0,
            expense_cents: 0,
            by_category: Vec::new(),
        });
    }

    fn toast(&mut self, message: String, color: Color32) {
        self.toasts.push(Toast {
            message,
            color,
            until: Instant::now() + Duration::from_secs(4),
        });
    }

    fn header(ui: &mut egui::Ui, title: &str) {
        ui.add_space(6.0);
        ui.label(RichText::new(title).size(24.0).strong().color(TEXT));
        ui.add_space(10.0);
    }

    fn gradient_title(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let title = "MON BUDGET";
            let palette = [YEL, AMB, ORA, REDO, RED];
            let chars: Vec<char> = title.chars().collect();
            let segments = palette.len();
            let per_segment = (chars.len() + segments - 1) / segments;
            for (index, chunk) in chars.chunks(per_segment).enumerate() {
                let text: String = chunk.iter().collect();
                ui.label(RichText::new(text).strong().size(20.0).color(palette[index]));
            }
        });
    }

    fn month_selector(ui: &mut egui::Ui, month: &mut String) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(RichText::new("Mois suivant >").color(TEXT)).clicked() {
                *month = shift_month(month, 1);
            }
            ui.label(RichText::new(month.as_str()).strong().size(20.0).color(YEL));
            if ui.button(RichText::new("< Mois précédent").color(TEXT)).clicked() {
                *month = shift_month(month, -1);
            }
        });
    }

    fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
        ui.group(|ui| {
            ui.set_min_size(Vec2::new(170.0, 60.0));
            ui.vertical(|ui| {
                ui.label(RichText::new(label).small().color(TEXT_DIM));
                ui.label(RichText::new(value).size(19.0).strong().color(color));
            });
        });
    }

    fn nav_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);
        Self::gradient_title(ui);
        ui.add_space(16.0);
        let pages = [
            (Page::Dashboard, "Tableau de bord"),
            (Page::Transactions, "Transactions"),
            (Page::Budgets, "Budgets"),
            (Page::Report, "Rapport mensuel"),
            (Page::Export, "Export CSV"),
        ];
        for (page, label) in pages {
            let selected = self.page == page;
            let button = egui::Button::new(
                RichText::new(label)
                    .strong()
                    .size(16.0)
                    .color(if selected { TEXT_DARK } else { TEXT }),
            )
            .fill(if selected { ORA } else { Color32::TRANSPARENT })
            .min_size(Vec2::new(ui.available_width(), 34.0));
            if ui.add(button).clicked() {
                self.page = page;
            }
            ui.add_space(4.0);
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new("SOLDE TOTAL").small().color(TEXT_DIM));
                ui.label(
                    RichText::new(euros(self.total_cents))
                        .strong()
                        .size(21.0)
                        .color(if self.total_cents < 0 { RED } else { ORA }),
                );
            });
            ui.add_space(8.0);
        });
    }

    fn dashboard(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            Self::header(ui, "Tableau de bord");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                Self::month_selector(ui, &mut self.month);
            });
        });
        ui.add_space(4.0);
        let net = self.summary.income_cents - self.summary.expense_cents;
        ui.horizontal(|ui| {
            Self::stat_card(ui, "Revenus du mois", &euros(self.summary.income_cents), AMB);
            Self::stat_card(ui, "Dépenses du mois", &euros(self.summary.expense_cents), REDO);
            Self::stat_card(
                ui,
                "Reste (épargne)",
                &euros(net),
                if net < 0 { RED } else { YEL },
            );
        });
        ui.add_space(14.0);
        ui.label(RichText::new("Mes comptes").strong().size(17.0).color(TEXT));
        ui.add_space(6.0);
        let account_cards = self.accounts.clone();
        ui.horizontal_wrapped(|ui| {
            for (account, balance) in &account_cards {
                ui.group(|ui| {
                    ui.set_min_size(Vec2::new(200.0, 58.0));
                    ui.vertical(|ui| {
                        ui.label(RichText::new(account.name.as_str()).strong().color(TEXT));
                        ui.label(RichText::new(account.kind.as_str()).small().color(TEXT_DIM));
                        ui.label(
                            RichText::new(euros(*balance))
                                .strong()
                                .color(if *balance < 0 { RED } else { YEL }),
                        );
                    });
                });
            }
            ui.group(|ui| {
                ui.set_min_size(Vec2::new(200.0, 58.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("+ Nouveau compte").strong().color(YEL),
                            ),
                        )
                        .clicked()
                    {
                        self.dialog = Some(Dialog::NewAccount(AccountForm::default()));
                    }
                });
            });
        });
        ui.add_space(14.0);
        ui.label(RichText::new("Dernières transactions").strong().size(17.0).color(TEXT));
        ui.add_space(6.0);
        if self.recent_txs.is_empty() {
            ui.label(
                RichText::new("Aucune transaction : commencez par enregistrer une dépense ou un revenu.")
                    .color(TEXT_DIM),
            );
        }
        ScrollArea::vertical().show(ui, |ui| {
            for row in &self.recent_txs {
                ui.group(|ui| {
                    ui.set_min_height(30.0);
                    transaction_row(ui, row);
                });
            }
        });
    }

    fn transactions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Transactions").size(24.0).strong().color(TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("+ Nouvelle transaction").strong())
                            .fill(ORA),
                    )
                    .clicked()
                {
                    self.dialog = Some(Dialog::NewTransaction(TxForm {
                        date: Local::now().naive_local().format("%Y-%m-%d").to_string(),
                        ..TxForm::default()
                    }));
                }
            });
        });
        ui.add_space(6.0);
        Self::month_selector(ui, &mut self.month);
        ui.add_space(10.0);
        if self.month_txs.is_empty() {
            ui.label(
                RichText::new("Aucune transaction ce mois-ci.")
                    .color(TEXT_DIM),
            );
        }
        ScrollArea::vertical().show(ui, |ui| {
            let rows = self.month_txs.clone();
            for row in &rows {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_height(30.0);
                        transaction_row(ui, row);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(RichText::new("Supprimer").small().color(RED))
                                .clicked()
                            {
                                self.confirm_delete = Some(row.id);
                            }
                            if ui.button(RichText::new("Modifier").small().color(AMB)).clicked() {
                                let form = TxForm {
                                    account_index: self.accounts.iter().position(|(account, _)| account.name == row.account).unwrap_or(0),
                                    category_index: self.categories.iter().position(|category| category.name == row.category).unwrap_or(0),
                                    amount: amount_input_text(row.amount_cents),
                                    date: row.date.clone(),
                                    note: row.note.clone(),
                                    error: None,
                                };
                                self.dialog = Some(Dialog::EditTransaction(row.id, form));
                            }
                        });
                    });
                });
                if self.confirm_delete == Some(row.id) {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Supprimer « {} {} » ?", row.date, row.category))
                                .color(RED)
                                .strong(),
                        );
                        if ui.add(egui::Button::new(RichText::new("Confirmer").color(TEXT_DARK)).fill(RED)).clicked() {
                            let _ = self.store.delete_transaction(row.id);
                            self.confirm_delete = None;
                            self.toast("Transaction supprimée.".into(), AMB);
                            self.refresh_data();
                        }
                        if ui.button("Annuler").clicked() {
                            self.confirm_delete = None;
                        }
                    });
                }
            }
        });
    }

    fn budgets_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Budgets du mois").size(24.0).strong().color(TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(egui::Button::new(RichText::new("+ Nouveau budget").strong()).fill(ORA))
                    .clicked()
                {
                    self.dialog = Some(Dialog::NewBudget(BudgetForm::default()));
                }
            });
        });
        ui.add_space(6.0);
        Self::month_selector(ui, &mut self.month);
        ui.add_space(10.0);
        if self.budgets.is_empty() {
            ui.label(
                RichText::new(
                    "Aucun budget défini pour ce mois. Fixez une limite par catégorie pour garder le contrôle.",
                )
                .color(TEXT_DIM),
            );
            return;
        }
        for line in &self.budgets {
            ui.group(|ui| {
                ui.set_min_height(64.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(line.category_name.as_str()).strong().color(REDO));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} / {}",
                                euros(line.spent_cents),
                                euros(line.limit_cents)
                            ))
                            .strong()
                            .color(TEXT),
                        );
                    });
                });
                let ratio = if line.limit_cents > 0 {
                    line.spent_cents as f32 / line.limit_cents as f32
                } else {
                    0.0
                };
                let percent = (ratio * 100.0).round() as i64;
                let color = if ratio > 1.0 {
                    RED
                } else if ratio >= 0.9 {
                    AMB
                } else {
                    ORA
                };
                ui.add(
                    egui::ProgressBar::new(ratio.clamp(0.0, 1.0))
                        .show_percentage()
                        .fill(color)
                        .desired_height(12.0),
                );
                if ratio > 1.0 {
                    ui.label(RichText::new("⚠ Budget dépassé !").strong().color(RED));
                } else if ratio >= 0.9 {
                    ui.label(RichText::new("⚠ Attention, proche de la limite").color(AMB));
                } else {
                    ui.label(
                        RichText::new(format!("Reste : {}", euros(line.limit_cents - line.spent_cents)))
                            .color(TEXT_DIM),
                    );
                }
                let _ = percent;
            });
        }
    }

    fn report(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            Self::header(ui, "Rapport mensuel");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                Self::month_selector(ui, &mut self.month);
            });
        });
        let net = self.summary.income_cents - self.summary.expense_cents;
        ui.horizontal(|ui| {
            Self::stat_card(ui, "Revenus", &euros(self.summary.income_cents), AMB);
            Self::stat_card(ui, "Dépenses", &euros(self.summary.expense_cents), REDO);
            Self::stat_card(ui, "Reste (épargne)", &euros(net), if net < 0 { RED } else { YEL });
            ui.group(|ui| {
                ui.set_min_size(Vec2::new(150.0, 60.0));
                ui.vertical(|ui| {
                    ui.label(RichText::new("Taux d'épargne").small().color(TEXT_DIM));
                    let rate = if self.summary.income_cents > 0 {
                        net as f32 / self.summary.income_cents as f32 * 100.0
                    } else {
                        0.0
                    };
                    ui.label(
                        RichText::new(format!("{rate:.1} %"))
                            .strong()
                            .size(19.0)
                            .color(if rate < 0.0 { RED } else { YEL }),
                    );
                });
            });
        });
        ui.add_space(16.0);
        ui.label(RichText::new("Dépenses par catégorie").strong().size(17.0).color(TEXT));
        ui.add_space(8.0);
        if self.summary.by_category.is_empty() {
            ui.label(RichText::new("Aucune dépense enregistrée ce mois-ci.").color(TEXT_DIM));
            return;
        }
        let max = self.summary.by_category[0].1.max(1) as f32;
        for (name, cents) in &self.summary.by_category {
            ui.label(RichText::new(name.as_str()).strong().color(REDO));
            ui.add(
                egui::ProgressBar::new(*cents as f32 / max)
                    .text(euros(*cents))
                    .fill(REDO)
                    .desired_height(14.0),
            );
            ui.add_space(4.0);
        }
    }

    fn export_page(&mut self, ui: &mut egui::Ui) {
        Self::header(ui, "Exporter en CSV");
        ui.label(
            RichText::new(
                "Génère un fichier CSV de toutes vos transactions, compatible Excel et LibreOffice Calc.",
            )
            .color(TEXT_DIM),
        );
        ui.add_space(16.0);
        if ui
            .add(
                egui::Button::new(
                    RichText::new("Exporter toutes les transactions").strong().size(16.0),
                )
                .fill(ORA)
                .min_size(Vec2::new(260.0, 40.0)),
            )
            .clicked()
        {
            match export_file(&self.store) {
                Ok(Some(path)) => self.toast(format!("✔ Export réussi : {path}"), AMB),
                Ok(None) => self.toast("Aucune transaction à exporter.".into(), TEXT_DIM),
                Err(error) => self.toast(format!("Erreur : {error}"), RED),
            }
        }
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        let mut dialog = match self.dialog.take() {
            Some(dialog) => dialog,
            None => return,
        };
        let mut close = false;
        match &mut dialog {
            Dialog::NewTransaction(form) => {
                egui::Window::new("Nouvelle transaction")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        transaction_form(ui, form, &self.accounts, &self.categories);
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.add(save_button()).clicked() && form.error.is_none() {
                                if let Some(cents) = validate_tx(form) {
                                    if let Some(account) = self.accounts.get(form.account_index) {
                                        if let Some(category) = self.categories.get(form.category_index) {
                                            let _ = self.store.add_transaction(
                                                account.0.id,
                                                category.id,
                                                cents,
                                                form.date.trim(),
                                                &form.note,
                                            );
                                            self.toast(
                                                format!(
                                                    "✔ {} de {} enregistré.",
                                                    if category.is_income() { "Revenu" } else { "Dépense" },
                                                    euros(cents)
                                                ),
                                                AMB,
                                            );
                                            close = true;
                                        }
                                    }
                                }
                            }
                            if ui.button("Annuler").clicked() {
                                close = true;
                            }
                        });
                    });
            }
            Dialog::EditTransaction(id, form) => {
                egui::Window::new("Modifier la transaction")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        transaction_form(ui, form, &self.accounts, &self.categories);
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.add(save_button()).clicked() && form.error.is_none() {
                                if let Some(cents) = validate_tx(form) {
                                if let Some(account) = self.accounts.get(form.account_index) {
                                    if let Some(category) = self.categories.get(form.category_index) {
                                            let _ = self.store.update_transaction(
                                                *id,
                                                account.0.id,
                                                category.id,
                                                cents,
                                                form.date.trim(),
                                                &form.note,
                                            );
                                            self.toast("✔ Transaction mise à jour.".into(), AMB);
                                            close = true;
                                        }
                                    }
                                }
                            }
                            if ui.button("Annuler").clicked() {
                                close = true;
                            }
                        });
                    });
            }
            Dialog::NewAccount(form) => {
                egui::Window::new("Nouveau compte")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        account_form(ui, form);
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.add(save_button()).clicked() && form.error.is_none() {
                                if let Some(cents) = parse_or_zero(&form.initial) {
                                    let kinds = ["Courant", "Épargne", "Espèces", "Autre"];
                                    match self.store.add_account(
                                        form.name.trim(),
                                        kinds[form.kind_index % kinds.len()],
                                        cents,
                                    ) {
                                        Ok(()) => {
                                            self.toast(format!("✔ Compte « {} » créé.", form.name.trim()), AMB);
                                            close = true;
                                        }
                                        Err(_) => {
                                            form.error = Some("Ce nom existe déjà.".into());
                                        }
                                    }
                                }
                            }
                            if ui.button("Annuler").clicked() {
                                close = true;
                            }
                        });
                    });
            }
            Dialog::NewBudget(form) => {
                egui::Window::new(format!("Budget de {month}", month = self.month))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        budget_form(ui, form, &self.categories);
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.add(save_button()).clicked() && form.error.is_none() {
                                let expense_categories: Vec<&Category> = self.categories.iter().filter(|category| !category.is_income()).collect();
                                if let Some(category) = expense_categories.get(form.category_index) {
                                    if let Some(cents) = parse_amount_cents(&form.limit) {
                                        let _ = self.store.set_budget(category.id, &self.month.clone(), cents);
                                        self.toast(format!("✔ Budget de {} fixé pour « {} ».", euros(cents), category.name), AMB);
                                        close = true;
                                    }
                                }
                            }
                            if ui.button("Annuler").clicked() {
                                close = true;
                            }
                        });
                    });
            }
        }
        self.dialog = if close { None } else { Some(dialog) };
        self.refresh_data();
    }

    fn show_toasts(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|toast| toast.until > Instant::now());
        egui::Area::new(egui::Id::new("toasts"))
            .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -20.0])
            .show(ctx, |ui| {
                for toast in &self.toasts {
                    ui.label(RichText::new(toast.message.as_str()).strong().color(toast.color));
                }
            });
    }
}

fn save_button() -> egui::Button<'static> {
    egui::Button::new(RichText::new("Enregistrer").strong().color(TEXT_DARK)).fill(ORA)
}

fn transaction_row(ui: &mut egui::Ui, row: &TransactionRow) {
    ui.label(RichText::new(row.date.as_str()).monospace().color(TEXT_DIM));
    ui.label(RichText::new(row.category.as_str()).strong().color(REDO));
    if row.note.is_empty() {
        ui.label(RichText::new("—").color(TEXT_DIM));
    } else {
        ui.label(RichText::new(row.note.as_str()).color(TEXT));
    }
    ui.label(RichText::new(format!("({})", row.account)).small().color(TEXT_DIM));
    let sign = if row.kind == "revenu" { "+" } else { "-" };
    ui.label(
        RichText::new(format!("{sign}{}", euros(row.amount_cents)))
            .strong()
            .color(if row.kind == "revenu" { AMB } else { REDO }),
    );
}

fn transaction_form(ui: &mut egui::Ui, form: &mut TxForm, accounts: &[(Account, i64)], categories: &[Category]) {
    if accounts.is_empty() {
        ui.label(RichText::new("⚠ Créez d'abord un compte (onglet Tableau de bord).").color(RED));
        return;
    }
    form.account_index = form.account_index.min(accounts.len() - 1);
    form.category_index = form.category_index.min(categories.len() - 1);
    ui.vertical(|ui| {
        ComboBox::from_label("Compte")
            .selected_text(accounts[form.account_index].0.name.as_str())
            .show_ui(ui, |ui| {
                for (index, (account, _)) in accounts.iter().enumerate() {
                    ui.selectable_value(&mut form.account_index, index, account.name.as_str());
                }
            });
        ComboBox::from_label("Catégorie")
            .selected_text(format!(
                "[{}] {}",
                if categories[form.category_index].is_income() { "Revenu" } else { "Dépense" },
                categories[form.category_index].name
            ))
            .show_ui(ui, |ui| {
                for (index, category) in categories.iter().enumerate() {
                    let label = format!(
                        "[{}] {}",
                        if category.is_income() { "Revenu" } else { "Dépense" },
                        category.name
                    );
                    ui.selectable_value(&mut form.category_index, index, label);
                }
            });
        ui.horizontal(|ui| {
            ui.label("Montant (€) :");
            if ui.add(egui::TextEdit::singleline(&mut form.amount).desired_width(120.0)).lost_focus() {
                if form.error.is_none() && parse_amount_cents(&form.amount).is_none() {
                    form.error = Some("Montant invalide, exemple : 12,50".into());
                }
            }
            ui.label("Date (AAAA-MM-JJ) :");
            ui.add(egui::TextEdit::singleline(&mut form.date).desired_width(120.0));
        });
        ui.horizontal(|ui| {
            ui.label("Note :");
            ui.add(egui::TextEdit::singleline(&mut form.note).desired_width(320.0));
        });
        if let Some(error) = &form.error {
            ui.label(RichText::new(error.as_str()).color(RED));
        }
    });
}

fn account_form(ui: &mut egui::Ui, form: &mut AccountForm) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("Nom du compte :");
            ui.add(egui::TextEdit::singleline(&mut form.name).desired_width(220.0));
        });
        ComboBox::from_label("Type")
            .selected_text(["Courant", "Épargne", "Espèces", "Autre"][form.kind_index % 4])
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut form.kind_index, 0, "Courant");
                ui.selectable_value(&mut form.kind_index, 1, "Épargne");
                ui.selectable_value(&mut form.kind_index, 2, "Espèces");
                ui.selectable_value(&mut form.kind_index, 3, "Autre");
            });
        ui.horizontal(|ui| {
            ui.label("Solde de départ (€) :");
            ui.add(egui::TextEdit::singleline(&mut form.initial).desired_width(120.0));
        });
        if let Some(error) = &form.error {
            ui.label(RichText::new(error.as_str()).color(RED));
        }
    });
}

fn budget_form(ui: &mut egui::Ui, form: &mut BudgetForm, categories: &[Category]) {
    let expense_categories: Vec<&Category> = categories.iter().filter(|category| !category.is_income()).collect();
    if expense_categories.is_empty() {
        return;
    }
    form.category_index = form.category_index.min(expense_categories.len() - 1);
    ui.vertical(|ui| {
        ComboBox::from_label("Catégorie")
            .selected_text(expense_categories[form.category_index].name.as_str())
            .show_ui(ui, |ui| {
                for (index, category) in expense_categories.iter().enumerate() {
                    ui.selectable_value(&mut form.category_index, index, category.name.as_str());
                }
            });
        ui.horizontal(|ui| {
            ui.label("Limite mensuelle (€) :");
            ui.add(egui::TextEdit::singleline(&mut form.limit).desired_width(120.0));
        });
        if let Some(error) = &form.error {
            ui.label(RichText::new(error.as_str()).color(RED));
        }
    });
}

fn parse_or_zero(raw: &str) -> Option<i64> {
    if raw.trim().is_empty() {
        return Some(0);
    }
    parse_amount_cents(raw)
}

impl eframe::App for FinanceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::setup_theme(ctx);
        self.refresh_data();
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(215.0)
            .show(ctx, |ui| {
                self.nav_panel(ui);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(18.0))
            .show(ctx, |ui| match self.page {
                Page::Dashboard => self.dashboard(ui),
                Page::Transactions => self.transactions(ui),
                Page::Budgets => self.budgets_page(ui),
                Page::Report => self.report(ui),
                Page::Export => self.export_page(ui),
            });
        self.show_dialog(ctx);
        self.show_toasts(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifts_months() {
        assert_eq!(shift_month("2026-11", 1), "2026-12");
        assert_eq!(shift_month("2026-11", -1), "2026-10");
        assert_eq!(shift_month("2026-12", 1), "2027-01");
        assert_eq!(shift_month("2026-01", -1), "2025-12");
    }

    #[test]
    fn parses_amounts() {
        assert_eq!(parse_amount_cents("12,50"), Some(1250));
        assert_eq!(parse_amount_cents(" 12.5 "), Some(1250));
        assert_eq!(parse_amount_cents("0"), None);
        assert_eq!(parse_amount_cents("xyz"), None);
    }

    #[test]
    fn checks_dates() {
        assert!(valid_date("2026-11-25"));
        assert!(!valid_date("25/11/2026"));
    }
}
