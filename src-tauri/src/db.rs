use rusqlite::Connection;
use std::env;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

fn fallback_app_data_dir() -> PathBuf {
    if let Ok(appdata) = env::var("APPDATA") {
        if !appdata.trim().is_empty() {
            return PathBuf::from(appdata).join("4chef");
        }
    }

    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        return PathBuf::from(home).join(".4chef");
    }

    env::temp_dir().join("4chef")
}

pub fn fallback_db_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("FOURCHEF_DB_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if let Some(parent) = candidate.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            return Ok(candidate);
        }
    }

    let dir = fallback_app_data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("4chef.db"))
}

pub fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| fallback_app_data_dir());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("4chef.db"))
}

pub fn open_db(path: &Path) -> Result<Connection, String> {
    Connection::open(path).map_err(|e| e.to_string())
}

pub fn open_initialized_db(app: &AppHandle) -> Result<Connection, String> {
    let path = db_path(app)?;
    let conn = open_db(&path)?;
    init_db(&conn)?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;

        CREATE TABLE IF NOT EXISTS units (
          unit_id INTEGER PRIMARY KEY,
          sing TEXT,
          plur TEXT,
          unit_type INTEGER,
          is_whole_unit INTEGER,
          unit_kind INTEGER
        );

        CREATE TABLE IF NOT EXISTS items (
          item_id INTEGER PRIMARY KEY,
          name TEXT,
          status INTEGER,
          raw_len INTEGER
        );

        CREATE TABLE IF NOT EXISTS vendors (
          vendor_id INTEGER PRIMARY KEY,
          name TEXT
        );

        CREATE TABLE IF NOT EXISTS recipes (
          recipe_id INTEGER PRIMARY KEY,
          recipe_group_id INTEGER,
          name TEXT,
          instructions TEXT
        );

        CREATE TABLE IF NOT EXISTS recp_items (
          recipe_id INTEGER,
          recp_item_id INTEGER,
          item_id INTEGER,
          unit_id INTEGER,
          qty REAL
        );

        CREATE TABLE IF NOT EXISTS convunit (
          item_id INTEGER,
          vendor_id INTEGER,
          unit_id1 INTEGER,
          unit_id2 INTEGER,
          qty1 REAL,
          qty2 REAL,
          status INTEGER,
          is_calculated INTEGER
        );

        CREATE TABLE IF NOT EXISTS inv_units (
          item_id INTEGER,
          purch_unit_id INTEGER,
          is_default INTEGER,
          status INTEGER
        );

        CREATE TABLE IF NOT EXISTS inv_prices (
          item_id INTEGER,
          vendor_id INTEGER,
          price REAL,
          pack TEXT,
          status INTEGER
        );

        CREATE TABLE IF NOT EXISTS conv_suggestions (
          item_id INTEGER,
          vendor_id INTEGER,
          unit_id1 INTEGER,
          unit_id2 INTEGER,
          qty1 REAL,
          qty2 REAL,
          recipe_unit TEXT,
          purch_unit TEXT,
          hits INTEGER,
          derived_from TEXT,
          hops INTEGER,
          path TEXT
        );

        CREATE TABLE IF NOT EXISTS conv_suggestions_safe (
          item_id INTEGER,
          vendor_id INTEGER,
          unit_id1 INTEGER,
          unit_id2 INTEGER,
          qty1 REAL,
          qty2 REAL,
          recipe_unit TEXT,
          purch_unit TEXT,
          hits INTEGER,
          derived_from TEXT,
          hops INTEGER,
          path TEXT
        );

        CREATE TABLE IF NOT EXISTS conv_todo (
          item_id INTEGER,
          vendor_id INTEGER,
          recipe_unit_id INTEGER,
          purch_unit_id INTEGER,
          recipe_unit TEXT,
          purch_unit TEXT,
          hits INTEGER,
          needed TEXT
        );

        CREATE TABLE IF NOT EXISTS missing_edges (
          item_id INTEGER,
          item_name TEXT,
          vendor_id INTEGER,
          recipe_unit_id INTEGER,
          recipe_unit TEXT,
          purch_unit_id INTEGER,
          purch_unit TEXT,
          hits INTEGER
        );

        CREATE TABLE IF NOT EXISTS missing_purch_unit (
          item_id INTEGER,
          item_name TEXT,
          usage_count INTEGER
        );

        CREATE TABLE IF NOT EXISTS missing_data_report (
          recipe_id INTEGER,
          recipe_name TEXT,
          missing_a INTEGER,
          missing_b INTEGER,
          missing_c INTEGER
        );

        CREATE TABLE IF NOT EXISTS invoices (
          status INTEGER,
          invoice_id INTEGER,
          invoice_date TEXT,
          vendor_id INTEGER,
          invoice_no TEXT,
          freight REAL,
          total REAL,
          col8 TEXT,
          col9 TEXT,
          col10 TEXT,
          col11 TEXT,
          col12 TEXT,
          col13 TEXT,
          col14 TEXT,
          col15 TEXT,
          col16 TEXT,
          col17 TEXT,
          col18 TEXT,
          col19 TEXT,
          col20 TEXT,
          col21 TEXT,
          col22 TEXT,
          col23 TEXT,
          col24 TEXT,
          col25 TEXT,
          col26 TEXT,
          col27 TEXT,
          col28 TEXT
        );

        CREATE TABLE IF NOT EXISTS trans (
          status INTEGER,
          invoice_id INTEGER,
          trans_id INTEGER,
          item_id INTEGER,
          trans_date TEXT,
          vendor_id INTEGER,
          price REAL,
          qty REAL,
          unit_id INTEGER,
          ext_cost REAL,
          col11 TEXT,
          col12 TEXT,
          col13 TEXT,
          col14 TEXT,
          col15 TEXT,
          col16 TEXT,
          col17 TEXT
        );

        CREATE TABLE IF NOT EXISTS recp_inv (
          recipe_id INTEGER,
          recp_inv_id INTEGER,
          item_id INTEGER,
          qty REAL,
          unit_id INTEGER,
          col6 TEXT,
          col7 TEXT,
          col8 TEXT,
          col9 TEXT,
          col10 TEXT,
          col11 TEXT
        );

        CREATE TABLE IF NOT EXISTS bids (
          col1 TEXT,
          col2 TEXT,
          col3 TEXT,
          col4 TEXT,
          col5 TEXT,
          col6 TEXT,
          col7 TEXT,
          col8 TEXT,
          col9 TEXT,
          col10 TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_convunit_item_vendor ON convunit(item_id, vendor_id);
        CREATE INDEX IF NOT EXISTS idx_recp_items_item ON recp_items(item_id);
        CREATE INDEX IF NOT EXISTS idx_invoices_vendor ON invoices(vendor_id);
        CREATE INDEX IF NOT EXISTS idx_trans_invoice ON trans(invoice_id);
        "#,
    )
    .map_err(|e| e.to_string())
    .and_then(
        |_| match conn.execute("ALTER TABLE recipes ADD COLUMN instructions TEXT", []) {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = err.to_string();
                if msg.contains("duplicate column name") {
                    Ok(())
                } else {
                    Err(msg)
                }
            }
        },
    )
    .and_then(
        |_| match conn.execute("ALTER TABLE inv_prices ADD COLUMN prev_price REAL", []) {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = err.to_string();
                if msg.contains("duplicate column name") {
                    Ok(())
                } else {
                    Err(msg)
                }
            }
        },
    )
    .and_then(
        |_| match conn.execute("ALTER TABLE items ADD COLUMN food_category TEXT", []) {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = err.to_string();
                if msg.contains("duplicate column name") {
                    Ok(())
                } else {
                    Err(msg)
                }
            }
        },
    )
    .and_then(
        |_| match conn.execute("ALTER TABLE items ADD COLUMN storage_type TEXT", []) {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = err.to_string();
                if msg.contains("duplicate column name") {
                    Ok(())
                } else {
                    Err(msg)
                }
            }
        },
    )
    .and_then(|_| {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
              key TEXT PRIMARY KEY,
              value TEXT
            );
            "#,
        )
        .map_err(|e| e.to_string())
    })
}

pub fn clear_tables(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        DELETE FROM units;
        DELETE FROM items;
        DELETE FROM vendors;
        DELETE FROM recipes;
        DELETE FROM recp_items;
        DELETE FROM convunit;
        DELETE FROM inv_units;
        DELETE FROM inv_prices;
        DELETE FROM conv_suggestions;
        DELETE FROM conv_suggestions_safe;
        DELETE FROM conv_todo;
        DELETE FROM missing_edges;
        DELETE FROM missing_purch_unit;
        DELETE FROM missing_data_report;
        DELETE FROM invoices;
        DELETE FROM trans;
        DELETE FROM recp_inv;
        DELETE FROM bids;
        "#,
    )
    .map_err(|e| e.to_string())
}

pub fn with_tx<T>(
    conn: &mut Connection,
    f: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let result = f(&tx)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
