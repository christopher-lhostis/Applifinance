//! Modèles de données de l'application.

#[derive(Debug, Clone)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub kind: String,
}

impl Category {
    pub fn is_income(&self) -> bool {
        self.kind == "revenu"
    }
}

#[derive(Debug, Clone)]
pub struct TransactionRow {
    pub id: i64,
    pub date: String,
    pub account: String,
    pub category: String,
    pub kind: String,
    pub amount_cents: i64,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct BudgetLine {
    pub category_id: i64,
    pub category_name: String,
    pub limit_cents: i64,
    pub spent_cents: i64,
}

#[derive(Debug, Clone)]
pub struct MonthSummary {
    pub income_cents: i64,
    pub expense_cents: i64,
    pub by_category: Vec<(String, i64)>,
}
