//! Serveur web local : l'interface graphique s'affiche dans le navigateur.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::export::export_csv_string;
use crate::store::Store;

const INDEX_HTML: &str = include_str!("../static/index.html");

pub fn serve(store: Store, port: u16) -> Result<()> {
    let address = format!("127.0.0.1:{port}");
    let server = tiny_http::Server::http(address.clone()).map_err(|error| anyhow!(error.to_string()))?;
    let url = format!("http://{address}");
    println!("✔ Mon Budget est prêt : {url}");
    println!("  (laissez cette fenêtre ouverte ; fermez-la ou Ctrl+C pour arrêter)");
    crate::web::open_browser(&url);
    for request in server.incoming_requests() {
        let _ = handle(request, &store);
    }
    Ok(())
}

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

fn handle(mut request: tiny_http::Request, store: &Store) -> Result<()> {
    let method = request.method().clone();
    let url = request.url().to_string();
    let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));

    match (&method, path) {
        (tiny_http::Method::Get, "/") => {
            let html = INDEX_HTML.to_string();
            respond(request, 200, html, "text/html; charset=utf-8", None);
        }
        (tiny_http::Method::Get, "/api/state") => {
            let params = parse_query(query);
            let month = params.get("month").cloned().unwrap_or_default();
            respond(request, 200, state_json(store, &month).to_string(), "application/json", None);
        }
        (tiny_http::Method::Post, "/api/accounts") => {
            let body = read_body(&mut request);
            respond(request, 200, add_account(store, &body).to_string(), "application/json", None);
        }
        (tiny_http::Method::Post, "/api/transactions") => {
            let body = read_body(&mut request);
            respond(request, 200, save_transaction(store, None, &body).to_string(), "application/json", None);
        }
        (tiny_http::Method::Put, path) if path.starts_with("/api/transactions/") => {
            let body = read_body(&mut request);
            let id: i64 = path.rsplit('/').next().and_then(|part| part.parse().ok()).unwrap_or(0);
            respond(request, 200, save_transaction(store, Some(id), &body).to_string(), "application/json", None);
        }
        (tiny_http::Method::Delete, path) if path.starts_with("/api/transactions/") => {
            let id: i64 = path.rsplit('/').next().and_then(|part| part.parse().ok()).unwrap_or(0);
            let deleted = store.delete_transaction(id).unwrap_or(false);
            respond(
                request,
                200,
                json!({ "ok": deleted, "error": if deleted { "" } else { "Transaction introuvable" } }).to_string(),
                "application/json",
                None,
            );
        }
        (tiny_http::Method::Post, "/api/budgets") => {
            let body = read_body(&mut request);
            respond(request, 200, set_budget(store, &body).to_string(), "application/json", None);
        }
        (tiny_http::Method::Delete, "/api/budgets") => {
            let params = parse_query(query);
            let category_id: i64 = params.get("category_id").and_then(|value| value.parse().ok()).unwrap_or(0);
            let month = params.get("month").cloned().unwrap_or_default();
            let deleted = store.delete_budget(category_id, &month).unwrap_or(false);
            respond(
                request,
                200,
                json!({ "ok": deleted, "error": if deleted { "" } else { "Budget introuvable" } }).to_string(),
                "application/json",
                None,
            );
        }
        (tiny_http::Method::Get, "/api/export") => match export_csv_string(store) {
            Ok(csv) => respond(
                request,
                200,
                csv,
                "text/csv; charset=utf-8",
                Some("attachment; filename=\"mes-transactions.csv\""),
            ),
            Err(error) => respond(
                request,
                500,
                json!({ "ok": false, "error": error.to_string() }).to_string(),
                "application/json",
                None,
            ),
        },
        _ => {
            respond(
                request,
                404,
                json!({ "ok": false, "error": "page introuvable" }).to_string(),
                "application/json",
                None,
            );
        }
    }
    Ok(())
}

fn state_json(store: &Store, month: &str) -> Value {
    let month = if month.len() == 7 { month.to_string() } else { crate::ui::current_month() };
    let accounts = store.accounts_with_balances().unwrap_or_default();
    let total_cents = store.total_balance().unwrap_or(0);
    let categories = store.categories().unwrap_or_default();
    let transactions = store.transactions_in_month(&month).unwrap_or_default();
    let budgets = store.budgets_for_month(&month).unwrap_or_default();
    let summary = store.month_summary(&month).unwrap();
    json!({
        "month": month,
        "accounts": accounts.iter().map(|(account, balance)| json!({
            "id": account.id,
            "name": account.name,
            "kind": account.kind,
            "balance_cents": balance,
        })).collect::<Vec<_>>(),
        "total_cents": total_cents,
        "categories": categories.iter().map(|category| json!({
            "id": category.id,
            "name": category.name,
            "kind": category.kind,
        })).collect::<Vec<_>>(),
        "transactions": transactions.iter().map(|row| json!({
            "id": row.id,
            "date": row.date,
            "account": row.account,
            "category": row.category,
            "kind": row.kind,
            "amount_cents": row.amount_cents,
            "note": row.note,
        })).collect::<Vec<_>>(),
        "budgets": budgets.iter().map(|line| json!({
            "category_id": line.category_id,
            "category_name": line.category_name,
            "limit_cents": line.limit_cents,
            "spent_cents": line.spent_cents,
        })).collect::<Vec<_>>(),
        "summary": {
            "income_cents": summary.income_cents,
            "expense_cents": summary.expense_cents,
            "by_category": summary.by_category.iter()
                .map(|(name, cents)| json!({ "name": name, "cents": cents }))
                .collect::<Vec<_>>(),
        }
    })
}

fn add_account(store: &Store, body: &str) -> Value {
    let Ok(data) = serde_json::from_str::<Value>(body) else {
        return json_error("Requête invalide");
    };
    let Some(name) = data["name"].as_str().map(str::trim).filter(|name| !name.is_empty()) else {
        return json_error("Le nom du compte est obligatoire.");
    };
    let kind = data["kind"].as_str().unwrap_or("Courant");
    let initial_cents = data["initial_cents"].as_i64().unwrap_or(0);
    match store.add_account(name, kind, initial_cents) {
        Ok(()) => {
            let id: i64 = store
                .list_accounts()
                .ok()
                .and_then(|accounts| accounts.iter().find(|account| account.name == name).map(|account| account.id))
                .unwrap_or(0);
            json!({ "ok": true, "id": id })
        }
        Err(_) => json_error("Un compte portant ce nom existe déjà."),
    }
}

fn save_transaction(store: &Store, id: Option<i64>, body: &str) -> Value {
    let Ok(data) = serde_json::from_str::<Value>(body) else {
        return json_error("Requête invalide");
    };
    let Some(account_id) = data["account_id"].as_i64() else {
        return json_error("Choisissez un compte.");
    };
    let Some(category_id) = data["category_id"].as_i64() else {
        return json_error("Choisissez une catégorie.");
    };
    let Some(cents) = data["amount_cents"].as_i64().filter(|cents| *cents > 0) else {
        return json_error("Montant invalide, exemple : 12,50");
    };
    let Some(date) = data["date"].as_str().filter(|date| crate::web::valid_date(date)) else {
        return json_error("Date invalide, format attendu AAAA-MM-JJ");
    };
    let note = data["note"].as_str().unwrap_or("");
    let result = match id {
        Some(id) => store.update_transaction(id, account_id, category_id, cents, date, note),
        None => store.add_transaction(account_id, category_id, cents, date, note).map(|_| true),
    };
    match result {
        Ok(_updated) => json!({ "ok": true }),
        Err(error) => json!({ "ok": false, "error": error.to_string() }),
    }
}

fn set_budget(store: &Store, body: &str) -> Value {
    let Ok(data) = serde_json::from_str::<Value>(body) else {
        return json_error("Requête invalide");
    };
    let Some(category_id) = data["category_id"].as_i64() else {
        return json_error("Choisissez une catégorie.");
    };
    let Some(month) = data["month"].as_str().filter(|month| month.len() == 7) else {
        return json_error("Mois invalide.");
    };
    let Some(limit) = data["limit_cents"].as_i64().filter(|limit| *limit > 0) else {
        return json_error("Limite invalide.");
    };
    match store.set_budget(category_id, month, limit) {
        Ok(()) => json!({ "ok": true }),
        Err(error) => json!({ "ok": false, "error": error.to_string() }),
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.insert(key.to_string(), value.to_string());
    }
    params
}

fn read_body(request: &mut tiny_http::Request) -> String {
    let mut body = String::new();
    let _ = request.as_reader().read_to_string(&mut body);
    body
}

fn valid_date(raw: &str) -> bool {
    chrono::NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").is_ok()
}

fn json_error(message: &str) -> Value {
    json!({ "ok": false, "error": message })
}

fn respond(request: tiny_http::Request, code: u16, body: String, content_type: &str, disposition: Option<&str>) {
    let mut response = tiny_http::Response::from_string(body).with_status_code(code);
    if let Ok(header) = tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        response = response.with_header(header);
    }
    if let Some(disposition) = disposition {
        if let Ok(header) = tiny_http::Header::from_bytes(&b"Content-Disposition"[..], disposition.as_bytes()) {
            response = response.with_header(header);
        }
    }
    let _ = request.respond(response);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_query() {
        let params = parse_query("month=2026-11&foo=");
        assert_eq!(params.get("month").map(String::as_str), Some("2026-11"));
        assert_eq!(params.get("foo"), Some(&String::new()));
    }

    #[test]
    fn validates_dates() {
        assert!(valid_date("2026-11-25"));
        assert!(!valid_date("25/11/2026"));
    }
}
