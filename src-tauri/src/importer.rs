use crate::db::{display_path, file_exists};
use csv::StringRecord;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;

const INV_BASE_COLS: usize = 32;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ImportSummary {
    pub db_path: String,
    pub units: usize,
    pub items: usize,
    pub convunit: usize,
    pub recp_items: usize,
    pub inv_units: usize,
    pub inv_prices: usize,
    pub vendors: usize,
    pub recipes: usize,
    pub conv_suggestions: usize,
    pub conv_suggestions_safe: usize,
    pub conv_todo: usize,
    pub missing_edges: usize,
    pub missing_purch_unit: usize,
    pub missing_data_report: usize,
    pub invoices: usize,
    pub trans: usize,
    pub recp_inv: usize,
    pub bids: usize,
    pub warnings: Vec<String>,
}

fn clean_field(value: &str) -> String {
    value.replace('\u{0}', "").trim().to_string()
}

fn parse_i64(value: Option<&str>) -> Option<i64> {
    value.and_then(|v| clean_field(v).parse().ok())
}

fn parse_f64(value: Option<&str>) -> Option<f64> {
    value.and_then(|v| clean_field(v).parse().ok())
}

fn is_header(record: &StringRecord, expected: &str) -> bool {
    record
        .get(0)
        .map(|v| clean_field(v).to_lowercase())
        .map(|v| v == expected)
        .unwrap_or(false)
}

pub fn import_all(conn: &Connection, base: &Path, db_path: &Path) -> Result<ImportSummary, String> {
    let mut summary = ImportSummary::default();
    summary.db_path = display_path(db_path);

    let units_path = base.join("Units.csv");
    if file_exists(&units_path) {
        summary.units = import_units(conn, &units_path, &mut summary.warnings)?;
    } else {
        summary.warnings.push(format!(
            "Missing Units.csv at {}",
            display_path(&units_path)
        ));
    }

    let inv_path = base.join("Inv.csv");
    if file_exists(&inv_path) {
        summary.items = import_inv(conn, &inv_path, &mut summary.warnings)?;
    } else {
        summary
            .warnings
            .push(format!("Missing Inv.csv at {}", display_path(&inv_path)));
    }

    let conv_path = base.join("ConvUnit.csv");
    if file_exists(&conv_path) {
        summary.convunit = import_convunit(conn, &conv_path, &mut summary.warnings)?;
    } else {
        summary.warnings.push(format!(
            "Missing ConvUnit.csv at {}",
            display_path(&conv_path)
        ));
    }

    let recp_items_path = base.join("RecpItems.csv");
    if file_exists(&recp_items_path) {
        summary.recp_items = import_recp_items(conn, &recp_items_path, &mut summary.warnings)?;
    } else {
        summary.warnings.push(format!(
            "Missing RecpItems.csv at {}",
            display_path(&recp_items_path)
        ));
    }

    let inv_units_path = base.join("InvUnits.csv");
    if file_exists(&inv_units_path) {
        summary.inv_units = import_inv_units(conn, &inv_units_path, &mut summary.warnings)?;
    } else {
        summary.warnings.push(format!(
            "Missing InvUnits.csv at {}",
            display_path(&inv_units_path)
        ));
    }

    let inv_prices_path = base.join("InvPrices.csv");
    if file_exists(&inv_prices_path) {
        summary.inv_prices = import_inv_prices(conn, &inv_prices_path, &mut summary.warnings)?;
    }

    let vendor_path = base.join("Vendor.csv");
    if file_exists(&vendor_path) {
        summary.vendors = import_vendors(conn, &vendor_path, &mut summary.warnings)?;
    }

    let recipe_path = base.join("Recipe.csv");
    if file_exists(&recipe_path) {
        summary.recipes = import_recipes(conn, &recipe_path, &mut summary.warnings)?;
    }

    let suggestions_path = base.join("convunit_suggestions.csv");
    if file_exists(&suggestions_path) {
        summary.conv_suggestions = import_conv_suggestions(
            conn,
            &suggestions_path,
            "conv_suggestions",
            &mut summary.warnings,
        )?;
    }

    let suggestions_safe_path = base.join("convunit_suggestions_safe.csv");
    if file_exists(&suggestions_safe_path) {
        summary.conv_suggestions_safe = import_conv_suggestions(
            conn,
            &suggestions_safe_path,
            "conv_suggestions_safe",
            &mut summary.warnings,
        )?;
    }

    let todo_path = base.join("convunit_todo.csv");
    if file_exists(&todo_path) {
        summary.conv_todo = import_conv_todo(conn, &todo_path, &mut summary.warnings)?;
    }

    let missing_edges_path = base.join("missing_edges.csv");
    if file_exists(&missing_edges_path) {
        summary.missing_edges =
            import_missing_edges(conn, &missing_edges_path, &mut summary.warnings)?;
    }

    let missing_purch_path = base.join("missing_purch_unit.csv");
    if file_exists(&missing_purch_path) {
        summary.missing_purch_unit =
            import_missing_purch_unit(conn, &missing_purch_path, &mut summary.warnings)?;
    }

    let missing_report_path = base.join("missing_data_report.csv");
    if file_exists(&missing_report_path) {
        summary.missing_data_report =
            import_missing_data_report(conn, &missing_report_path, &mut summary.warnings)?;
    }

    let invoice_path = base.join("Invoice.csv");
    if file_exists(&invoice_path) {
        summary.invoices = import_invoices(conn, &invoice_path, &mut summary.warnings)?;
    }

    let trans_path = base.join("Trans.csv");
    if file_exists(&trans_path) {
        summary.trans = import_trans(conn, &trans_path, &mut summary.warnings)?;
    }

    let recp_inv_path = base.join("RecpInv.csv");
    if file_exists(&recp_inv_path) {
        summary.recp_inv = import_recp_inv(conn, &recp_inv_path, &mut summary.warnings)?;
    }

    let bids_path = base.join("Bids.csv");
    if file_exists(&bids_path) {
        summary.bids = import_bids(conn, &bids_path, &mut summary.warnings)?;
    }

    if summary.inv_prices == 0 && summary.trans > 0 {
        let added = backfill_prices_from_trans(conn)?;
        if added > 0 {
            summary.inv_prices = added;
            summary.warnings.push(format!(
                "InvPrices.csv was empty; backfilled {} prices from Trans.csv",
                added
            ));
        } else {
            summary
                .warnings
                .push("InvPrices.csv empty and no prices found in Trans.csv".to_string());
        }
    }

    Ok(summary)
}

fn import_units(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO units (unit_id, sing, plur, unit_type, is_whole_unit, unit_kind)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() == 0 {
            continue;
        }
        if is_header(&record, "unitid") {
            continue;
        }
        let unit_id = parse_i64(record.get(0));
        if unit_id.is_none() {
            warnings.push(format!(
                "Units.csv: skipped row with invalid UnitID: {:?}",
                record
            ));
            continue;
        }
        let sing = record.get(1).map(clean_field).unwrap_or_default();
        let plur = record.get(2).map(clean_field).unwrap_or_default();
        let unit_type = parse_i64(record.get(3));
        let is_whole = parse_i64(record.get(4));
        let unit_kind = parse_i64(record.get(5));

        stmt.execute((unit_id, sing, plur, unit_type, is_whole, unit_kind))
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_inv(
    conn: &Connection,
    path: &Path,
    _warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO items (item_id, name, status, raw_len) VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 3 {
            continue;
        }
        let status = parse_i64(record.get(0));
        let item_id = parse_i64(record.get(1));
        if item_id.is_none() {
            continue;
        }

        let extra_cols = record.len().saturating_sub(INV_BASE_COLS);
        let name_end = 2 + extra_cols + 1;
        let mut name_parts = Vec::new();
        for idx in 2..name_end.min(record.len()) {
            let part = clean_field(record.get(idx).unwrap_or(""));
            if !part.is_empty() {
                name_parts.push(part);
            }
        }
        let name = name_parts.join(", ");

        stmt.execute((item_id, name, status, record.len() as i64))
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_convunit(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO convunit (item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, status, is_calculated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() == 0 {
            continue;
        }
        if is_header(&record, "itemid") {
            continue;
        }
        let item_id = parse_i64(record.get(0));
        let vendor_id = parse_i64(record.get(1)).unwrap_or(0);
        let unit_id1 = parse_i64(record.get(2));
        let unit_id2 = parse_i64(record.get(3));
        let qty1 = parse_f64(record.get(4));
        let qty2 = parse_f64(record.get(5));
        if item_id.is_none()
            || unit_id1.is_none()
            || unit_id2.is_none()
            || qty1.is_none()
            || qty2.is_none()
        {
            warnings.push(format!("ConvUnit.csv: skipped row {:?}", record));
            continue;
        }
        let status = parse_i64(record.get(6));
        let is_calc = parse_i64(record.get(7));

        stmt.execute((
            item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, status, is_calc,
        ))
        .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_recp_items(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO recp_items (recipe_id, recp_item_id, item_id, unit_id, qty)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 5 {
            continue;
        }
        // RecpItems.csv order: RecpItemID, RecipeID, ItemID, UnitID, Qty, ...
        let recp_item_id = parse_i64(record.get(0));
        let recipe_id = parse_i64(record.get(1));
        let item_id = parse_i64(record.get(2));
        let unit_id = parse_i64(record.get(3));
        let qty = parse_f64(record.get(4));
        if recipe_id.is_none() || item_id.is_none() || unit_id.is_none() || qty.is_none() {
            warnings.push(format!("RecpItems.csv: skipped row {:?}", record));
            continue;
        }

        stmt.execute((recipe_id, recp_item_id, item_id, unit_id, qty))
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_inv_units(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO inv_units (item_id, purch_unit_id, is_default, status)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 4 {
            continue;
        }
        let status = parse_i64(record.get(0));
        let item_id = parse_i64(record.get(1));
        let purch_unit_id = parse_i64(record.get(2));
        let is_default = parse_i64(record.get(3));
        if item_id.is_none() || purch_unit_id.is_none() {
            warnings.push(format!("InvUnits.csv: skipped row {:?}", record));
            continue;
        }
        stmt.execute((item_id, purch_unit_id, is_default, status))
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_inv_prices(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO inv_prices (item_id, vendor_id, price, pack, status)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 5 {
            continue;
        }
        let status = parse_i64(record.get(0));
        let item_id = parse_i64(record.get(1));
        let vendor_id = parse_i64(record.get(2)).unwrap_or(0);
        let price = parse_f64(record.get(3));
        let pack = record.get(4).map(clean_field).unwrap_or_default();
        if item_id.is_none() {
            warnings.push(format!("InvPrices.csv: skipped row {:?}", record));
            continue;
        }

        stmt.execute((item_id, vendor_id, price, pack, status))
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_vendors(
    conn: &Connection,
    path: &Path,
    _warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("INSERT OR REPLACE INTO vendors (vendor_id, name) VALUES (?1, ?2)")
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 4 {
            continue;
        }
        let vendor_id = parse_i64(record.get(0));
        if vendor_id.is_none() {
            continue;
        }
        let name = record.get(3).map(clean_field).unwrap_or_default();
        stmt.execute((vendor_id, name)).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_recipes(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut stmt = conn
        .prepare("INSERT OR REPLACE INTO recipes (recipe_id, recipe_group_id, name, instructions) VALUES (?1, ?2, ?3, ?4)")
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    let contents = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut current = String::new();
    let mut merged_lines = 0usize;
    let mut records: Vec<String> = Vec::new();

    for raw_line in contents.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }

        if looks_like_recipe_row_start(line) {
            if !current.is_empty() {
                records.push(current.clone());
                current.clear();
            }
            current.push_str(line);
        } else {
            if current.is_empty() {
                continue;
            }
            merged_lines += 1;
            current.push(' ');
            current.push_str(line);
        }
    }
    if !current.is_empty() {
        records.push(current);
    }

    for record in records {
        if let Some(parsed) = parse_recipe_record(&record) {
            let (recipe_id, group_id, name, instructions) = parsed;
            if name.is_empty() {
                continue;
            }
            stmt.execute((recipe_id, group_id, name, instructions))
                .map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    if merged_lines > 0 {
        warnings.push(format!(
            "Recipe.csv contained multi-line RTF; merged {} continuation lines and stored plain-text instructions",
            merged_lines
        ));
    }

    Ok(count)
}

fn import_conv_suggestions(
    conn: &Connection,
    path: &Path,
    table: &str,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let sql = format!(
        "INSERT INTO {} (item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, recipe_unit, purch_unit, hits, derived_from, hops, path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        table
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut count = 0usize;

    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 12 {
            warnings.push(format!("{}: skipped short row {:?}", table, record));
            continue;
        }
        let item_id = parse_i64(record.get(0));
        let vendor_id = parse_i64(record.get(1)).unwrap_or(0);
        let unit_id1 = parse_i64(record.get(2));
        let unit_id2 = parse_i64(record.get(3));
        let qty1 = parse_f64(record.get(4));
        let qty2 = parse_f64(record.get(5));
        let recipe_unit = record.get(6).map(clean_field).unwrap_or_default();
        let purch_unit = record.get(7).map(clean_field).unwrap_or_default();
        let hits = parse_i64(record.get(8));
        let derived_from = record.get(9).map(clean_field).unwrap_or_default();
        let hops = parse_i64(record.get(10));
        let path = record.get(11).map(clean_field).unwrap_or_default();

        if item_id.is_none()
            || unit_id1.is_none()
            || unit_id2.is_none()
            || qty1.is_none()
            || qty2.is_none()
        {
            warnings.push(format!("{}: skipped row {:?}", table, record));
            continue;
        }

        stmt.execute((
            item_id,
            vendor_id,
            unit_id1,
            unit_id2,
            qty1,
            qty2,
            recipe_unit,
            purch_unit,
            hits,
            derived_from,
            hops,
            path,
        ))
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    Ok(count)
}

fn import_conv_todo(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO conv_todo (item_id, vendor_id, recipe_unit_id, purch_unit_id, recipe_unit, purch_unit, hits, needed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 8 {
            warnings.push(format!("conv_todo: skipped short row {:?}", record));
            continue;
        }
        let item_id = parse_i64(record.get(0));
        let vendor_id = parse_i64(record.get(1)).unwrap_or(0);
        let recipe_unit_id = parse_i64(record.get(2));
        let purch_unit_id = parse_i64(record.get(3));
        let recipe_unit = record.get(4).map(clean_field).unwrap_or_default();
        let purch_unit = record.get(5).map(clean_field).unwrap_or_default();
        let hits = parse_i64(record.get(6));
        let needed = record.get(7).map(clean_field).unwrap_or_default();

        if item_id.is_none() || recipe_unit_id.is_none() || purch_unit_id.is_none() {
            warnings.push(format!("conv_todo: skipped row {:?}", record));
            continue;
        }

        stmt.execute((
            item_id,
            vendor_id,
            recipe_unit_id,
            purch_unit_id,
            recipe_unit,
            purch_unit,
            hits,
            needed,
        ))
        .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_missing_edges(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO missing_edges (item_id, item_name, vendor_id, recipe_unit_id, recipe_unit, purch_unit_id, purch_unit, hits)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 8 {
            warnings.push(format!("missing_edges: skipped short row {:?}", record));
            continue;
        }
        let item_id = parse_i64(record.get(0));
        let item_name = record.get(1).map(clean_field).unwrap_or_default();
        let vendor_id = parse_i64(record.get(2)).unwrap_or(0);
        let recipe_unit_id = parse_i64(record.get(3));
        let recipe_unit = record.get(4).map(clean_field).unwrap_or_default();
        let purch_unit_id = parse_i64(record.get(5));
        let purch_unit = record.get(6).map(clean_field).unwrap_or_default();
        let hits = parse_i64(record.get(7));

        if item_id.is_none() || recipe_unit_id.is_none() || purch_unit_id.is_none() {
            warnings.push(format!("missing_edges: skipped row {:?}", record));
            continue;
        }

        stmt.execute((
            item_id,
            item_name,
            vendor_id,
            recipe_unit_id,
            recipe_unit,
            purch_unit_id,
            purch_unit,
            hits,
        ))
        .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_missing_purch_unit(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO missing_purch_unit (item_id, item_name, usage_count)
             VALUES (?1, ?2, ?3)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 3 {
            warnings.push(format!(
                "missing_purch_unit: skipped short row {:?}",
                record
            ));
            continue;
        }
        let item_id = parse_i64(record.get(0));
        let item_name = record.get(1).map(clean_field).unwrap_or_default();
        let usage_count = parse_i64(record.get(2));

        if item_id.is_none() {
            warnings.push(format!("missing_purch_unit: skipped row {:?}", record));
            continue;
        }

        stmt.execute((item_id, item_name, usage_count))
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_missing_data_report(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO missing_data_report (recipe_id, recipe_name, missing_a, missing_b, missing_c)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 4 {
            continue;
        }
        let recipe_id = parse_i64(record.get(0));
        if recipe_id.is_none() {
            warnings.push(format!("missing_data_report: skipped row {:?}", record));
            continue;
        }

        if record.len() < 5 {
            warnings.push(format!("missing_data_report: short row {:?}", record));
            continue;
        }

        let tail = record.len();
        let missing_a = parse_i64(record.get(tail - 3));
        let missing_b = parse_i64(record.get(tail - 2));
        let missing_c = parse_i64(record.get(tail - 1));
        let name_parts = record
            .iter()
            .skip(1)
            .take(tail.saturating_sub(4))
            .map(clean_field)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        let recipe_name = name_parts.join(", ");

        stmt.execute((recipe_id, recipe_name, missing_a, missing_b, missing_c))
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

pub fn import_invoices_and_trans(
    conn: &Connection,
    invoice_path: &Path,
    trans_path: &Path,
    warnings: &mut Vec<String>,
) -> Result<(usize, usize), String> {
    let mut invoice_ids = Vec::new();
    collect_invoice_ids(invoice_path, &mut invoice_ids)?;
    collect_invoice_ids(trans_path, &mut invoice_ids)?;

    if !invoice_ids.is_empty() {
        invoice_ids.sort();
        invoice_ids.dedup();

        let placeholders = std::iter::repeat("?")
            .take(invoice_ids.len())
            .collect::<Vec<_>>()
            .join(", ");

        let delete_trans = format!("DELETE FROM trans WHERE invoice_id IN ({})", placeholders);
        let delete_invoices = format!("DELETE FROM invoices WHERE invoice_id IN ({})", placeholders);

        conn.execute(&delete_trans, rusqlite::params_from_iter(invoice_ids.iter()))
            .map_err(|e| e.to_string())?;
        conn.execute(&delete_invoices, rusqlite::params_from_iter(invoice_ids.iter()))
            .map_err(|e| e.to_string())?;
    }

    let invoices = import_invoices(conn, invoice_path, warnings)?;
    let trans = import_trans(conn, trans_path, warnings)?;
    Ok((invoices, trans))
}

fn collect_invoice_ids(path: &Path, out: &mut Vec<i64>) -> Result<(), String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if let Some(invoice_id) = parse_i64(record.get(1)) {
            out.push(invoice_id);
        }
    }

    Ok(())
}

fn import_invoices(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO invoices (status, invoice_id, invoice_date, vendor_id, invoice_no, freight, total, col8, col9, col10, col11, col12, col13, col14, col15, col16, col17, col18, col19, col20, col21, col22, col23, col24, col25, col26, col27, col28)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 7 {
            continue;
        }
        let status = parse_i64(record.get(0));
        let invoice_id = parse_i64(record.get(1));
        let invoice_date = record.get(2).map(clean_field).unwrap_or_default();
        let vendor_id = parse_i64(record.get(3));
        let invoice_no = record.get(4).map(clean_field).unwrap_or_default();
        let freight = parse_f64(record.get(5));
        let total = parse_f64(record.get(6));

        let mut cols = vec![String::new(); 21];
        for (idx, col) in record.iter().skip(7).take(21).enumerate() {
            cols[idx] = clean_field(col);
        }

        stmt.execute(rusqlite::params![
            status,
            invoice_id,
            invoice_date,
            vendor_id,
            invoice_no,
            freight,
            total,
            cols[0].clone(),
            cols[1].clone(),
            cols[2].clone(),
            cols[3].clone(),
            cols[4].clone(),
            cols[5].clone(),
            cols[6].clone(),
            cols[7].clone(),
            cols[8].clone(),
            cols[9].clone(),
            cols[10].clone(),
            cols[11].clone(),
            cols[12].clone(),
            cols[13].clone(),
            cols[14].clone(),
            cols[15].clone(),
            cols[16].clone(),
            cols[17].clone(),
            cols[18].clone(),
            cols[19].clone(),
            cols[20].clone(),
        ])
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    if count == 0 {
        warnings.push("Invoice.csv contained no importable rows".to_string());
    }

    Ok(count)
}

fn import_trans(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO trans (status, invoice_id, trans_id, item_id, trans_date, vendor_id, price, qty, unit_id, ext_cost, col11, col12, col13, col14, col15, col16, col17)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 9 {
            continue;
        }
        let status = parse_i64(record.get(0));
        let invoice_id = parse_i64(record.get(1));
        let trans_id = parse_i64(record.get(2));
        let item_id = parse_i64(record.get(3));
        let trans_date = record.get(4).map(clean_field).unwrap_or_default();
        let vendor_id = parse_i64(record.get(5));
        let price = parse_f64(record.get(6));
        let qty = parse_f64(record.get(7));
        let unit_id = parse_i64(record.get(8));
        let ext_cost = parse_f64(record.get(9));

        let mut cols = vec![String::new(); 7];
        for (idx, col) in record.iter().skip(10).take(7).enumerate() {
            cols[idx] = clean_field(col);
        }

        stmt.execute(rusqlite::params![
            status,
            invoice_id,
            trans_id,
            item_id,
            trans_date,
            vendor_id,
            price,
            qty,
            unit_id,
            ext_cost,
            cols[0].clone(),
            cols[1].clone(),
            cols[2].clone(),
            cols[3].clone(),
            cols[4].clone(),
            cols[5].clone(),
            cols[6].clone(),
        ])
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    if count == 0 {
        warnings.push("Trans.csv contained no importable rows".to_string());
    }

    Ok(count)
}

fn import_recp_inv(
    conn: &Connection,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO recp_inv (recipe_id, recp_inv_id, item_id, qty, unit_id, col6, col7, col8, col9, col10, col11)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() < 5 {
            continue;
        }
        let recipe_id = parse_i64(record.get(0));
        let recp_inv_id = parse_i64(record.get(1));
        let item_id = parse_i64(record.get(2));
        let qty = parse_f64(record.get(3));
        let unit_id = parse_i64(record.get(4));
        if recipe_id.is_none() || item_id.is_none() {
            warnings.push(format!("RecpInv.csv: skipped row {:?}", record));
            continue;
        }
        let mut cols = vec![String::new(); 6];
        for (idx, col) in record.iter().skip(5).take(6).enumerate() {
            cols[idx] = clean_field(col);
        }

        stmt.execute((
            recipe_id,
            recp_inv_id,
            item_id,
            qty,
            unit_id,
            cols[0].clone(),
            cols[1].clone(),
            cols[2].clone(),
            cols[3].clone(),
            cols[4].clone(),
            cols[5].clone(),
        ))
        .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn import_bids(
    conn: &Connection,
    path: &Path,
    _warnings: &mut Vec<String>,
) -> Result<usize, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .quoting(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT INTO bids (col1, col2, col3, col4, col5, col6, col7, col8, col9, col10)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        if record.len() == 0 {
            continue;
        }
        let mut cols = vec![String::new(); 10];
        for (idx, col) in record.iter().take(10).enumerate() {
            cols[idx] = clean_field(col);
        }
        stmt.execute((
            cols[0].clone(),
            cols[1].clone(),
            cols[2].clone(),
            cols[3].clone(),
            cols[4].clone(),
            cols[5].clone(),
            cols[6].clone(),
            cols[7].clone(),
            cols[8].clone(),
            cols[9].clone(),
        ))
        .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn backfill_prices_from_trans(conn: &Connection) -> Result<usize, String> {
    let sql = r#"
        INSERT INTO inv_prices (item_id, vendor_id, price, pack, status)
        SELECT t.item_id, t.vendor_id, t.price, '', 1
        FROM trans t
        INNER JOIN (
            SELECT item_id, vendor_id, MAX(COALESCE(trans_date, '')) AS max_date
            FROM trans
            WHERE price IS NOT NULL AND price > 0
            GROUP BY item_id, vendor_id
        ) latest
        ON t.item_id = latest.item_id
        AND t.vendor_id = latest.vendor_id
        AND COALESCE(t.trans_date, '') = latest.max_date
        WHERE t.price IS NOT NULL AND t.price > 0
    "#;

    conn.execute(sql, [])
        .map_err(|e| e.to_string())
        .map(|count| count as usize)
}

fn looks_like_recipe_row_start(line: &str) -> bool {
    let mut iter = line.splitn(3, ',');
    if let (Some(a), Some(b), Some(_)) = (iter.next(), iter.next(), iter.next()) {
        return a.trim().chars().all(|c| c.is_ascii_digit())
            && b.trim().chars().all(|c| c.is_ascii_digit());
    }
    false
}

fn parse_recipe_record(record: &str) -> Option<(i64, Option<i64>, String, String)> {
    let mut parts = record.split(',');
    let _status = parts.next()?.trim();
    let recipe_id_raw = parts.next()?.trim();
    let recipe_id = parse_i64(Some(recipe_id_raw))?;

    let rest: Vec<&str> = parts.collect();
    if rest.is_empty() {
        return None;
    }

    let mut group_idx = None;
    let mut group_id = None;
    for (i, token) in rest.iter().enumerate() {
        if let Some(val) = parse_i64(Some(token.trim())) {
            group_idx = Some(i);
            group_id = Some(val);
            break;
        }
    }

    let name = if let Some(idx) = group_idx {
        rest[..idx].join(",")
    } else {
        rest.join(",")
    };
    let name = clean_field(&name);

    let instructions = if let Some(pos) = record.find("{\\rtf") {
        let stripped = strip_rtf_groups(&record[pos..]);
        let text = rtf_to_text(&stripped);
        cleanup_instruction_text(text)
    } else {
        String::new()
    };

    Some((recipe_id, group_id, name, instructions))
}

fn rtf_to_text(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' | '}' => {}
            '\\' => match chars.peek() {
                Some('\\') => {
                    out.push('\\');
                    chars.next();
                }
                Some('{') => {
                    out.push('{');
                    chars.next();
                }
                Some('}') => {
                    out.push('}');
                    chars.next();
                }
                Some('~') => {
                    out.push(' ');
                    chars.next();
                }
                Some('\'') => {
                    chars.next();
                    let h1 = chars.next();
                    let h2 = chars.next();
                    if let (Some(h1), Some(h2)) = (h1, h2) {
                        let hex = format!("{}{}", h1, h2);
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            if byte.is_ascii_graphic() || byte == b' ' {
                                out.push(byte as char);
                            } else {
                                out.push(' ');
                            }
                        }
                    }
                }
                Some('u') => {
                    chars.next();
                    let mut num = String::new();
                    if let Some(&ch) = chars.peek() {
                        if ch == '-' || ch.is_ascii_digit() {
                            num.push(chars.next().unwrap());
                        }
                    }
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() {
                            num.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if let Ok(val) = num.parse::<i32>() {
                        if let Some(ch) = std::char::from_u32(val as u32) {
                            if ch.is_ascii() {
                                out.push(ch);
                            } else {
                                out.push(' ');
                            }
                        }
                    }
                    if let Some(&'?') = chars.peek() {
                        chars.next();
                    }
                }
                Some(_) => {
                    let mut word = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphabetic() {
                            word.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if let Some(&ch) = chars.peek() {
                        if ch == '-' || ch.is_ascii_digit() {
                            chars.next();
                            while let Some(&ch2) = chars.peek() {
                                if ch2.is_ascii_digit() {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    if let Some(&' ') = chars.peek() {
                        chars.next();
                    }
                    if word == "par" || word == "line" {
                        out.push('\n');
                    } else if word == "tab" {
                        out.push(' ');
                    }
                }
                None => {}
            },
            '\n' | '\r' => out.push(' '),
            _ => out.push(c),
        }
    }

    out
}

fn strip_rtf_groups(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::new();
    let mut depth = 0usize;
    let mut ignore_stack: Vec<usize> = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '{' {
            depth += 1;
            // Look ahead to identify group header
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let lookahead = &input[j..input.len().min(j + 32)];
            let header = lookahead.trim_start();
            if header.starts_with("\\fonttbl")
                || header.starts_with("\\colortbl")
                || header.starts_with("\\stylesheet")
                || header.starts_with("\\info")
                || header.starts_with("\\generator")
                || header.starts_with("\\header")
                || header.starts_with("\\footer")
                || header.starts_with("\\*")
            {
                ignore_stack.push(depth);
            }

            if ignore_stack.is_empty() {
                out.push(c);
            }
        } else if c == '}' {
            if let Some(&top) = ignore_stack.last() {
                if depth == top {
                    ignore_stack.pop();
                }
            }
            if ignore_stack.is_empty() {
                out.push(c);
            }
            if depth > 0 {
                depth -= 1;
            }
        } else {
            if ignore_stack.is_empty() {
                out.push(c);
            }
        }
        i += 1;
    }

    out
}

fn cleanup_instruction_text(mut text: String) -> String {
    text = text.trim().to_string();
    if text.is_empty() {
        return text;
    }

    if let Some(idx) = find_numbered_start(&text) {
        let prefix = &text[..idx];
        if should_trim_rtf_prefix(prefix) {
            text = text[idx..].to_string();
        }
    }

    normalize_instruction_text(text)
}

fn find_numbered_start(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        let c = bytes[i] as char;
        let next = bytes[i + 1] as char;
        if c == '1' && (next == ')' || next == '.' || next == ':') {
            if i == 0
                || text[..i]
                    .chars()
                    .last()
                    .map(|ch| ch.is_whitespace())
                    .unwrap_or(false)
            {
                return Some(i);
            }
        }
    }
    None
}

fn normalize_instruction_text(text: String) -> String {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let normalized = normalize_spaces(raw);
        let cleaned = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
        let cleaned = strip_control_prefix(&cleaned);
        if !cleaned.is_empty() {
            lines.push(cleaned);
        }
    }

    let mut joined = lines.join("\n");
    if joined.is_empty() {
        return joined;
    }

    joined = insert_numbered_breaks(&joined);
    joined
}

fn strip_control_prefix(line: &str) -> String {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return String::new();
    }

    let mut idx = 0usize;
    while idx < tokens.len() && is_rtf_control_token(tokens[idx]) {
        idx += 1;
    }

    if idx >= tokens.len() {
        String::new()
    } else {
        tokens[idx..].join(" ")
    }
}

fn normalize_spaces(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            '\u{00A0}' | '\u{2007}' | '\u{202F}' => ' ',
            _ => ch,
        })
        .collect()
}

fn is_rtf_control_token(token: &str) -> bool {
    let lower = token.to_lowercase();
    if !lower.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return false;
    }

    let mut letters = String::new();
    let mut digits = 0usize;
    let mut saw_digit = false;
    for ch in lower.chars() {
        if !saw_digit {
            if ch.is_ascii_alphabetic() {
                letters.push(ch);
            } else if ch.is_ascii_digit() {
                saw_digit = true;
                digits += 1;
            } else {
                return false;
            }
        } else if ch.is_ascii_digit() {
            digits += 1;
        } else {
            return false;
        }
    }

    if letters.is_empty() || digits == 0 {
        return false;
    }

    matches!(
        letters.as_str(),
        "c" | "cf"
            | "f"
            | "fs"
            | "lang"
            | "highlight"
            | "ul"
            | "fi"
            | "li"
            | "ri"
            | "sb"
            | "sa"
            | "q"
            | "qc"
            | "ql"
            | "qr"
            | "qj"
    )
}

fn should_trim_rtf_prefix(prefix: &str) -> bool {
    let lower = prefix.to_lowercase();
    if lower.contains("riched")
        || lower.contains("rtf")
        || lower.contains("fonttbl")
        || lower.contains("colortbl")
        || lower.contains("tahoma")
        || lower.contains("arial")
        || lower.contains("times")
        || lower.contains("calibri")
        || lower.contains("californian")
    {
        return true;
    }

    if prefix.contains('\\') || prefix.contains('{') || prefix.contains('}') {
        return true;
    }

    if prefix.contains(';') {
        return true;
    }

    prefix
        .split_whitespace()
        .any(|token| is_rtf_control_token(token))
}

fn insert_numbered_breaks(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::new();
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_digit() {
            let start = i;
            let mut j = i;
            while j < bytes.len() && (bytes[j] as char).is_ascii_digit() {
                j += 1;
            }
            if j < bytes.len() {
                let next = bytes[j] as char;
                let next2 = if j + 1 < bytes.len() {
                    bytes[j + 1] as char
                } else {
                    '\0'
                };
                let prev = if start > 0 {
                    bytes[start - 1] as char
                } else {
                    '\n'
                };
                if (next == ')' || (next == '.' && next2 == ' '))
                    && prev != '\n'
                    && (prev.is_whitespace() || is_sentence_break(prev))
                {
                    while out.ends_with(' ') {
                        out.pop();
                    }
                    out.push('\n');
                }
            }
            out.push_str(&text[start..j]);
            i = j;
            continue;
        }
        out.push(c);
        i += 1;
    }

    out
}

fn is_sentence_break(ch: char) -> bool {
    matches!(ch, '.' | ':' | ';' | ')' | ']' | '>')
}
