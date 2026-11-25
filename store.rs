//! Accès à la base SQLite locale (fichier unique, aucun serveur requis).

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::{Account, BudgetLine, Category, MonthSummary, TransactionRow};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL,
    initial_balance_cents INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('revenu', 'depense'))
);
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
    date TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS budgets (
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    month TEXT NOT NULL,
    limit_cents INTEGER NOT NULL,
    PRIMARY KEY (category_id, month)
);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
";

const DEFAULT_CATEGORIES: &[(&str, &str)] = &[
    ("Salaire", "revenu"),
    ("Remboursement / Aide", "revenu"),
    ("Autre revenu", "revenu"),
    ("Alimentation", "depense"),
    ("Loyer / Logement", "depense"),
    ("Factures", "depense"),
    ("Transport", "depense"),
    ("Loisirs", "depense"),
    ("Abonnements", "depense"),
    ("Shopping", "depense"),
    ("Santé", "depense"),
    ("Épargne", "depense"),
    ("Autre dépense", "depense"),
];

pub struct Store {
    conn: Connection,
}

const SELECT_TRANSACTION_ROWS: &str = "
    SELECT t.id, t.date, a.name, c.name, c.kind, t.amount_cents, t.note
    FROM transactions t
    JOIN accounts a ON a.id = t.account_id
    JOIN categories c ON c.id = t.category_id
";

fn row_to_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<TransactionRow> {
    Ok(TransactionRow {
        id: row.get(0)?,
        date: row.get(1)?,
        account: row.get(2)?,
        category: row.get(3)?,
        kind: row.get(4)?,
        amount_cents: row.get(5)?,
        note: row.get(6)?,
    })
}

impl Store {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Impossible d'ouvrir la base de données « {path} »"))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        let store = Self { conn };
        store.seed_default_categories()?;
        Ok(store)
    }

    fn seed_default_categories(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
        if count == 0 {
            for (name, kind) in DEFAULT_CATEGORIES {
                self.conn.execute(
                    "INSERT INTO categories (name, kind) VALUES (?1, ?2)",
                    params![name, kind],
                )?;
            }
        }
        Ok(())
    }

    pub fn count_accounts(&self) -> Result<i64> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn list_accounts(&self) -> Result<Vec<Account>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, kind FROM accounts ORDER BY id")?;
        let accounts = stmt
            .query_map([], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(accounts)
    }

    pub fn accounts_with_balances(&self) -> Result<Vec<(Account, i64)>> {
        let accounts = self.list_accounts()?;
        let mut result = Vec::new();
        for account in accounts {
            let balance = self.account_balance(account.id)?;
            result.push((account, balance));
        }
        Ok(result)
    }

    pub fn account_balance(&self, account_id: i64) -> Result<i64> {
        let initial: i64 = self.conn.query_row(
            "SELECT initial_balance_cents FROM accounts WHERE id = ?1",
            params![account_id],
            |row| row.get(0),
        )?;
        let delta: i64 = self.conn.query_row(
            "
            SELECT
                COALESCE(SUM(CASE WHEN c.kind = 'revenu' THEN t.amount_cents ELSE 0 END), 0)
                - COALESCE(SUM(CASE WHEN c.kind = 'depense' THEN t.amount_cents ELSE 0 END), 0)
            FROM transactions t
            JOIN categories c ON c.id = t.category_id
            WHERE t.account_id = ?1
            ",
            params![account_id],
            |row| row.get(0),
        )?;
        Ok(initial + delta)
    }

    pub fn total_balance(&self) -> Result<i64> {
        let mut total = 0;
        for account in self.list_accounts()? {
            total += self.account_balance(account.id)?;
        }
        Ok(total)
    }

    pub fn add_account(&self, name: &str, kind: &str, initial_cents: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO accounts (name, kind, initial_balance_cents) VALUES (?1, ?2, ?3)",
            params![name, kind, initial_cents],
        )?;
        Ok(())
    }

    pub fn delete_account(&self, id: i64) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    pub fn categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, kind FROM categories ORDER BY kind, name")?;
        let categories = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }

    pub fn add_transaction(
        &self,
        account_id: i64,
        category_id: i64,
        cents: i64,
        date: &str,
        note: &str,
    ) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO transactions (account_id, category_id, amount_cents, date, note)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![account_id, category_id, cents, date, note],
        )?;
        Ok(())
    }

    pub fn last_transactions(&self, limit: i64) -> Result<Vec<TransactionRow>> {
        let sql = format!(
            "{SELECT_TRANSACTION_ROWS} ORDER BY t.date DESC, t.id DESC LIMIT ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params![limit], row_to_transaction)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn transactions_in_month(&self, month: &str) -> Result<Vec<TransactionRow>> {
        let pattern = format!("{month}%");
        let sql = format!(
            "{SELECT_TRANSACTION_ROWS} WHERE t.date LIKE ?1 ORDER BY t.date, t.id"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params![pattern], row_to_transaction)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn all_transactions(&self) -> Result<Vec<TransactionRow>> {
        let sql = format!("{SELECT_TRANSACTION_ROWS} ORDER BY t.date, t.id");
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], row_to_transaction)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn count_transactions(&self) -> Result<i64> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM transactions", [], |row| row.get(0))?;
        Ok(count)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_transaction(
        &self,
        id: i64,
        account_id: i64,
        category_id: i64,
        cents: i64,
        date: &str,
        note: &str,
    ) -> Result<bool> {
        let updated = self.conn.execute(
            "
            UPDATE transactions
            SET account_id = ?1, category_id = ?2, amount_cents = ?3, date = ?4, note = ?5
            WHERE id = ?6
            ",
            params![account_id, category_id, cents, date, note, id],
        )?;
        Ok(updated > 0)
    }

    pub fn delete_transaction(&self, id: i64) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM transactions WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    pub fn count_transactions_for_account(&self, id: i64) -> Result<i64> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE account_id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn set_budget(&self, category_id: i64, month: &str, limit_cents: i64) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO budgets (category_id, month, limit_cents)
            VALUES (?1, ?2, ?3)
            ON CONFLICT (category_id, month)
            DO UPDATE SET limit_cents = excluded.limit_cents
            ",
            params![category_id, month, limit_cents],
        )?;
        Ok(())
    }

    pub fn delete_budget(&self, category_id: i64, month: &str) -> Result<bool> {
        let deleted = self.conn.execute(
            "DELETE FROM budgets WHERE category_id = ?1 AND month = ?2",
            params![category_id, month],
        )?;
        Ok(deleted > 0)
    }

    pub fn budgets_for_month(&self, month: &str) -> Result<Vec<BudgetLine>> {
        let pattern = format!("{month}%");
        let mut stmt = self.conn.prepare(
            "
            SELECT b.category_id, c.name, b.limit_cents,
                   COALESCE((
                       SELECT SUM(t.amount_cents)
                       FROM transactions t
                       WHERE t.category_id = b.category_id AND t.date LIKE ?1
                   ), 0) AS spent
            FROM budgets b
            JOIN categories c ON c.id = b.category_id
            WHERE b.month = ?2
            ORDER BY c.name
            ",
        )?;
        let lines = stmt
            .query_map(params![pattern, month], |row| {
                Ok(BudgetLine {
                    category_id: row.get(0)?,
                    category_name: row.get(1)?,
                    limit_cents: row.get(2)?,
                    spent_cents: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(lines)
    }

    pub fn month_summary(&self, month: &str) -> Result<MonthSummary> {
        let pattern = format!("{month}%");
        let mut totals = MonthSummary {
            income_cents: 0,
            expense_cents: 0,
            by_category: Vec::new(),
        };
        {
            let mut stmt = self.conn.prepare(
                "
                SELECT c.kind, COALESCE(SUM(t.amount_cents), 0)
                FROM transactions t
                JOIN categories c ON c.id = t.category_id
                WHERE t.date LIKE ?1
                GROUP BY c.kind
                ",
            )?;
            let mut rows = stmt.query(params![pattern])?;
            while let Some(row) = rows.next()? {
                let kind: String = row.get(0)?;
                let amount: i64 = row.get(1)?;
                if kind == "revenu" {
                    totals.income_cents = amount;
                } else {
                    totals.expense_cents = amount;
                }
            }
        }
        let mut stmt = self.conn.prepare(
            "
            SELECT c.name, COALESCE(SUM(t.amount_cents), 0) AS total
            FROM transactions t
            JOIN categories c ON c.id = t.category_id
            WHERE t.date LIKE ?1 AND c.kind = 'depense'
            GROUP BY c.name
            ORDER BY total DESC
            ",
        )?;
        totals.by_category = stmt
            .query_map(params![pattern], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(totals)
    }
}
