use std::env;
use std::path::{Path, PathBuf};

use rusqlite::OptionalExtension;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
mod db;
mod importer;

use db::{clear_tables, db_path, fallback_db_path, init_db, open_db, open_initialized_db, with_tx};
use importer::{import_all, import_invoices_and_trans, ImportSummary};

#[derive(Serialize)]
struct DbPathResponse {
    path: String,
}

#[derive(Serialize)]
struct PingResponse {
    ok: bool,
    message: String,
}

#[derive(Serialize)]
struct InventoryItem {
    item_id: i64,
    name: String,
    status: Option<i64>,
}

#[derive(Serialize)]
struct InventoryQueryResponse {
    items: Vec<InventoryItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize)]
struct InventoryDetailUnit {
    unit_id: Option<i64>,
    unit_name: String,
    is_default: Option<i64>,
    status: Option<i64>,
}

#[derive(Serialize)]
struct InventoryDetailPrice {
    vendor_id: Option<i64>,
    vendor_name: String,
    price: Option<f64>,
    pack: String,
    status: Option<i64>,
}

#[derive(Serialize)]
struct InventoryDetailConversion {
    vendor_id: i64,
    unit_id1: i64,
    unit_id2: i64,
    qty1: f64,
    qty2: f64,
}

#[derive(Serialize)]
struct InventoryDetailUsage {
    recipe_id: i64,
    recipe_name: String,
    qty: Option<f64>,
    unit_name: String,
}

#[derive(Serialize)]
struct InventoryDetailMissingEdge {
    vendor_id: i64,
    recipe_unit: String,
    purch_unit: String,
    hits: Option<i64>,
}

#[derive(Serialize)]
struct InventoryDetailResponse {
    item_id: i64,
    name: String,
    status: Option<i64>,
    purch_units: Vec<InventoryDetailUnit>,
    prices: Vec<InventoryDetailPrice>,
    conversions: Vec<InventoryDetailConversion>,
    usage: Vec<InventoryDetailUsage>,
    missing_edges: Vec<InventoryDetailMissingEdge>,
}

#[derive(Serialize, Clone)]
struct VendorListItem {
    vendor_id: i64,
    name: String,
    price_items: i64,
}

#[derive(Serialize)]
struct VendorListResponse {
    vendors: Vec<VendorListItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize)]
struct VendorPriceItem {
    item_id: i64,
    item_name: String,
    price: Option<f64>,
    pack: String,
}

#[derive(Serialize)]
struct VendorDetailResponse {
    vendor_id: i64,
    name: String,
    price_items: Vec<VendorPriceItem>,
    invoice_count: i64,
    trans_count: i64,
}

#[derive(Serialize)]
struct VendorSimple {
    vendor_id: i64,
    name: String,
}

#[derive(Serialize)]
struct VendorSimpleResponse {
    vendors: Vec<VendorSimple>,
}

#[derive(Serialize)]
struct UnitSimple {
    unit_id: i64,
    sing: String,
}

#[derive(Serialize)]
struct UnitSimpleResponse {
    units: Vec<UnitSimple>,
}

#[derive(Serialize)]
struct ItemSimple {
    item_id: i64,
    name: String,
}

#[derive(Serialize)]
struct ItemSimpleResponse {
    items: Vec<ItemSimple>,
}

#[derive(Serialize)]
struct RecipeListItem {
    recipe_id: i64,
    name: String,
    item_count: i64,
}

#[derive(Serialize)]
struct RecipeListResponse {
    recipes: Vec<RecipeListItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize)]
struct RecipeIngredient {
    recp_item_id: i64,
    item_id: i64,
    item_name: String,
    unit_id: Option<i64>,
    unit_name: String,
    qty: Option<f64>,
    purch_unit_id: Option<i64>,
    purch_unit_name: String,
    price: Option<f64>,
    extended_cost: Option<f64>,
    cost_status: String,
}

#[derive(Serialize)]
struct RecipeDetailResponse {
    recipe_id: i64,
    name: String,
    item_count: i64,
    total_cost: f64,
    missing_costs: i64,
    instructions: String,
    ingredients: Vec<RecipeIngredient>,
}

#[derive(Serialize)]
struct ConversionSuggestionRow {
    item_id: i64,
    vendor_id: i64,
    unit_id1: i64,
    unit_id2: i64,
    qty1: f64,
    qty2: f64,
    recipe_unit: String,
    purch_unit: String,
    hits: Option<i64>,
    derived_from: String,
    hops: Option<i64>,
    path: String,
}

#[derive(Serialize)]
struct ConversionSuggestionResponse {
    rows: Vec<ConversionSuggestionRow>,
    total: i64,
}

#[derive(Serialize)]
struct ConversionTodoRow {
    item_id: i64,
    vendor_id: i64,
    recipe_unit_id: i64,
    purch_unit_id: i64,
    recipe_unit: String,
    purch_unit: String,
    hits: Option<i64>,
    needed: String,
}

#[derive(Serialize)]
struct ConversionTodoResponse {
    rows: Vec<ConversionTodoRow>,
    total: i64,
}

#[derive(Serialize)]
struct MissingEdgeRow {
    item_id: i64,
    item_name: String,
    vendor_id: i64,
    recipe_unit_id: i64,
    recipe_unit: String,
    purch_unit_id: i64,
    purch_unit: String,
    hits: Option<i64>,
}

#[derive(Serialize)]
struct MissingEdgeResponse {
    rows: Vec<MissingEdgeRow>,
    total: i64,
}

#[derive(Serialize)]
struct MissingPurchRow {
    item_id: i64,
    item_name: String,
    usage_count: Option<i64>,
}

#[derive(Serialize)]
struct MissingPurchResponse {
    rows: Vec<MissingPurchRow>,
    total: i64,
}

#[derive(Serialize)]
struct MissingDataRow {
    recipe_id: i64,
    recipe_name: String,
    missing_a: Option<i64>,
    missing_b: Option<i64>,
    missing_c: Option<i64>,
}

#[derive(Serialize)]
struct MissingDataResponse {
    rows: Vec<MissingDataRow>,
    total: i64,
}

#[derive(Serialize)]
struct ConversionOverview {
    suggestions: i64,
    suggestions_safe: i64,
    todo: i64,
    missing_edges: i64,
    missing_purch: i64,
    missing_data: i64,
}

#[derive(Serialize)]
struct InvoiceListItem {
    invoice_id: i64,
    invoice_no: String,
    vendor_id: Option<i64>,
    vendor_name: String,
    invoice_date: String,
    total: Option<f64>,
}

#[derive(Serialize)]
struct InvoiceListResponse {
    invoices: Vec<InvoiceListItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize)]
struct InvoiceLineItem {
    trans_id: Option<i64>,
    item_id: Option<i64>,
    item_name: String,
    qty: Option<f64>,
    unit_name: String,
    price: Option<f64>,
    ext_cost: Option<f64>,
}

#[derive(Serialize)]
struct InvoiceDetailResponse {
    invoice: InvoiceListItem,
    freight: Option<f64>,
    lines: Vec<InvoiceLineItem>,
}

#[derive(Serialize)]
struct ExportCsvResponse {
    path: String,
    rows: usize,
}

#[derive(Serialize)]
struct PatchResponse {
    ok: bool,
    message: String,
}

const REQUIRED_EXPORT_FILES: [&str; 4] = ["Units.csv", "Inv.csv", "ConvUnit.csv", "RecpItems.csv"];
const MDF_EXPORT_TABLES: [(&str, &str); 12] = [
    ("Units", "Units.csv"),
    ("Inv", "Inv.csv"),
    ("ConvUnit", "ConvUnit.csv"),
    ("RecpItems", "RecpItems.csv"),
    ("InvUnits", "InvUnits.csv"),
    ("InvPrices", "InvPrices.csv"),
    ("Vendor", "Vendor.csv"),
    ("Recipe", "Recipe.csv"),
    ("Invoice", "Invoice.csv"),
    ("Trans", "Trans.csv"),
    ("RecpInv", "RecpInv.csv"),
    ("Bids", "Bids.csv"),
];

fn user_home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

fn fallback_data_root() -> PathBuf {
    if let Ok(appdata) = env::var("APPDATA") {
        if !appdata.trim().is_empty() {
            return PathBuf::from(appdata).join("4chef");
        }
    }

    if let Some(home) = user_home_dir() {
        return home.join(".4chef");
    }

    env::temp_dir().join("4chef")
}

fn open_in_file_browser(target: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = target;
        return Err("Opening paths is not supported on this platform".to_string());
    }
}

fn expand_base_path(input: &str) -> String {
    let mut path_str = input.to_string();

    if let Some(home) = user_home_dir() {
        if path_str == "~" {
            path_str = home.to_string_lossy().to_string();
        } else if path_str.starts_with("~/") || path_str.starts_with("~\\") {
            path_str = home.join(&path_str[2..]).to_string_lossy().to_string();
        }

        let home_str = home.to_string_lossy();
        path_str = path_str.replace("${HOME}", home_str.as_ref());
        path_str = path_str.replace("$HOME", home_str.as_ref());
        path_str = path_str.replace("%USERPROFILE%", home_str.as_ref());
    }

    path_str
}

fn table_count(conn: &rusqlite::Connection, table: &str) -> usize {
    conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
        row.get::<_, i64>(0)
    })
    .ok()
    .and_then(|count| usize::try_from(count).ok())
    .unwrap_or(0)
}

fn has_required_exports_dir(path: &Path) -> bool {
    REQUIRED_EXPORT_FILES.iter().all(|name| path.join(name).exists())
}

fn import_from_base_dir(app: &tauri::AppHandle, base: &Path) -> Result<ImportSummary, String> {
    if !base.exists() {
        return Err(format!("Base path does not exist: {}", base.display()));
    }
    if !base.is_dir() {
        return Err(format!("Base path is not a directory: {}", base.display()));
    }

    let db_path = db_path(app)?;
    let mut conn = open_db(&db_path)?;
    init_db(&conn)?;

    with_tx(&mut conn, |tx| {
        clear_tables(tx)?;
        import_all(tx, base, &db_path)
    })
}

fn is_ldf_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("ldf"))
        .unwrap_or(false)
}

fn find_first_mdf_in_dir(dir: &Path) -> Option<PathBuf> {
    let mut candidate = None;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_mdf_file(&path) {
                candidate = Some(path);
                break;
            }
        }
    }
    candidate
}

fn find_matching_ldf_for_mdf(mdf: &Path) -> Option<PathBuf> {
    let parent = mdf.parent()?;
    let stem = mdf.file_stem()?.to_string_lossy();
    let ldf = parent.join(format!("{stem}.ldf"));
    if ldf.exists() {
        Some(ldf)
    } else {
        None
    }
}

fn find_matching_mdf_for_ldf(ldf: &Path) -> Option<PathBuf> {
    let parent = ldf.parent()?;
    let stem = ldf.file_stem()?.to_string_lossy();
    let mdf = parent.join(format!("{stem}.mdf"));
    if mdf.exists() {
        return Some(mdf);
    }
    find_first_mdf_in_dir(parent)
}

fn path_has_binary(path: &Path) -> bool {
    path.is_file()
}

fn command_in_path(name: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };
    for dir in env::split_paths(&path_var) {
        let unix = dir.join(name);
        if path_has_binary(&unix) {
            return true;
        }
        #[cfg(target_os = "windows")]
        {
            for ext in ["exe", "cmd", "bat"] {
                let win = dir.join(format!("{name}.{ext}"));
                if path_has_binary(&win) {
                    return true;
                }
            }
        }
    }
    false
}

fn sql_escape_literal(input: &str) -> String {
    input.replace('\'', "''")
}

fn sql_server_name() -> String {
    if let Ok(server) = env::var("FOURCHEF_MSSQL_SERVER") {
        let trimmed = server.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    #[cfg(target_os = "windows")]
    {
        "(localdb)\\MSSQLLocalDB".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "localhost".to_string()
    }
}

fn maybe_start_localdb(server: &str) -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Ok(());
    }
    if !server.eq_ignore_ascii_case("(localdb)\\MSSQLLocalDB") {
        return Ok(());
    }
    if !command_in_path("sqllocaldb") {
        return Ok(());
    }

    let output = Command::new("sqllocaldb")
        .arg("start")
        .arg("MSSQLLocalDB")
        .output()
        .map_err(|e| format!("Failed to start LocalDB: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "Failed to start LocalDB MSSQLLocalDB: {} {}",
            stdout.trim(),
            stderr.trim()
        ))
    }
}

fn run_sqlcmd(
    server: &str,
    database: &str,
    query: &str,
    output_file: Option<&Path>,
) -> Result<String, String> {
    if !command_in_path("sqlcmd") {
        return Err(
            "sqlcmd is not installed. Install Microsoft sqlcmd tools and (on Windows) SQL Server LocalDB or set FOURCHEF_MSSQL_SERVER."
                .to_string(),
        );
    }

    let mut cmd = Command::new("sqlcmd");
    cmd.arg("-S")
        .arg(server)
        .arg("-d")
        .arg(database)
        .arg("-b")
        .arg("-W")
        .arg("-h")
        .arg("-1")
        .arg("-s")
        .arg(",")
        .arg("-w")
        .arg("65535")
        .arg("-y")
        .arg("0")
        .arg("-Y")
        .arg("0")
        .arg("-Q")
        .arg(query);

    if let Some(path) = output_file {
        cmd.arg("-o").arg(path);
    }

    let user = env::var("FOURCHEF_MSSQL_USER")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let pass = env::var("FOURCHEF_MSSQL_PASSWORD")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    match (user, pass) {
        (Some(u), Some(p)) => {
            cmd.arg("-U").arg(u).arg("-P").arg(p);
        }
        (Some(_), None) => {
            return Err(
                "FOURCHEF_MSSQL_USER is set but FOURCHEF_MSSQL_PASSWORD is empty.".to_string(),
            );
        }
        _ => {
            cmd.arg("-E");
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run sqlcmd: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "sqlcmd failed (server={}, db={}): {} {}",
            server,
            database,
            stdout.trim(),
            stderr.trim()
        ))
    }
}

fn parse_sqlcmd_int(output: &str) -> Option<i64> {
    output
        .lines()
        .map(str::trim)
        .find_map(|line| line.parse::<i64>().ok())
}

fn sql_table_exists(server: &str, database: &str, table: &str) -> Result<bool, String> {
    let query = format!(
        "SET NOCOUNT ON; SELECT CASE WHEN OBJECT_ID(N'dbo.{table}', N'U') IS NULL THEN 0 ELSE 1 END;"
    );
    let output = run_sqlcmd(server, database, &query, None)?;
    Ok(parse_sqlcmd_int(&output).unwrap_or(0) == 1)
}

fn export_table_csv_with_sqlcmd(
    server: &str,
    database: &str,
    table: &str,
    out_path: &Path,
) -> Result<(), String> {
    let query = format!("SET NOCOUNT ON; SELECT * FROM [dbo].[{table}];");
    run_sqlcmd(server, database, &query, Some(out_path)).map(|_| ())
}

fn build_export_dir() -> Result<PathBuf, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let dir = fallback_data_root()
        .join("imports")
        .join(format!("mdf_export_{ts}"));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn export_mdf_to_csv_dir(
    mdf: &Path,
    ldf: Option<&Path>,
    out_dir: &Path,
) -> Result<Vec<String>, String> {
    let server = sql_server_name();
    maybe_start_localdb(&server)?;

    let db_name = format!(
        "fourchef_import_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs()
    );
    let db_name_escaped = sql_escape_literal(&db_name);
    let mdf_escaped = sql_escape_literal(&mdf.to_string_lossy());
    let attach_sql = if let Some(ldf_path) = ldf {
        let ldf_escaped = sql_escape_literal(&ldf_path.to_string_lossy());
        format!(
            "IF DB_ID(N'{db_name_escaped}') IS NOT NULL BEGIN ALTER DATABASE [{db_name}] SET SINGLE_USER WITH ROLLBACK IMMEDIATE; EXEC sp_detach_db N'{db_name_escaped}'; END; CREATE DATABASE [{db_name}] ON (FILENAME=N'{mdf_escaped}'), (FILENAME=N'{ldf_escaped}') FOR ATTACH;"
        )
    } else {
        format!(
            "IF DB_ID(N'{db_name_escaped}') IS NOT NULL BEGIN ALTER DATABASE [{db_name}] SET SINGLE_USER WITH ROLLBACK IMMEDIATE; EXEC sp_detach_db N'{db_name_escaped}'; END; CREATE DATABASE [{db_name}] ON (FILENAME=N'{mdf_escaped}') FOR ATTACH_REBUILD_LOG;"
        )
    };

    run_sqlcmd(&server, "master", &attach_sql, None)?;

    let mut warnings = Vec::new();
    let mut export_error: Option<String> = None;
    for (table, file_name) in MDF_EXPORT_TABLES {
        match sql_table_exists(&server, &db_name, table) {
            Ok(true) => {
                let out = out_dir.join(file_name);
                if let Err(err) = export_table_csv_with_sqlcmd(&server, &db_name, table, &out) {
                    export_error = Some(format!("Failed exporting {table} to {}: {err}", out.display()));
                    break;
                }
            }
            Ok(false) => warnings.push(format!("MDF table dbo.{table} was not found")),
            Err(err) => {
                export_error = Some(format!("Failed checking table dbo.{table}: {err}"));
                break;
            }
        }
    }

    let detach_sql = format!(
        "IF DB_ID(N'{db_name_escaped}') IS NOT NULL BEGIN ALTER DATABASE [{db_name}] SET SINGLE_USER WITH ROLLBACK IMMEDIATE; EXEC sp_detach_db N'{db_name_escaped}'; END;"
    );
    let detach_result = run_sqlcmd(&server, "master", &detach_sql, None);

    if let Some(err) = export_error {
        if let Err(detach_err) = detach_result {
            return Err(format!("{err}. Also failed to detach temp DB: {detach_err}"));
        }
        return Err(err);
    }

    if let Err(detach_err) = detach_result {
        warnings.push(format!("Detached with warning: {detach_err}"));
    }

    Ok(warnings)
}

fn import_from_mdf_pair(
    app: &tauri::AppHandle,
    mdf: &Path,
    ldf: Option<&Path>,
) -> Result<ImportSummary, String> {
    let export_dir = build_export_dir()?;
    let mut export_warnings = export_mdf_to_csv_dir(mdf, ldf, &export_dir)?;
    let mut summary = import_from_base_dir(app, &export_dir)?;
    summary.warnings.append(&mut export_warnings);
    summary
        .warnings
        .push(format!("Imported from MDF source {} via SQL export", mdf.display()));
    summary.warnings.push(format!(
        "Staged export files saved at {}",
        export_dir.display()
    ));
    Ok(summary)
}

#[tauri::command]
fn open_path(path: String) -> Result<PatchResponse, String> {
    if path.trim().is_empty() {
        return Err("Path is empty".to_string());
    }
    let expanded = expand_base_path(path.trim());
    let p = std::path::Path::new(&expanded);
    let target = if p.is_dir() {
        expanded
    } else if let Some(parent) = p.parent() {
        if parent.is_dir() {
            parent.to_string_lossy().to_string()
        } else {
            expanded
        }
    } else {
        expanded
    };

    open_in_file_browser(&target)?;

    Ok(PatchResponse {
        ok: true,
        message: format!("Opened {}", target),
    })
}

#[tauri::command]
fn get_db_path(app: tauri::AppHandle) -> Result<DbPathResponse, String> {
    let path = db_path(&app)?;
    Ok(DbPathResponse {
        path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn import_exports(app: tauri::AppHandle, base_path: String) -> Result<ImportSummary, String> {
    println!("import_exports called with base_path={base_path}");
    let base = resolve_base_path(&base_path)?;
    import_from_base_dir(&app, &base)
}

#[tauri::command]
fn get_import_summary(app: tauri::AppHandle) -> Result<ImportSummary, String> {
    let conn = open_initialized_db(&app)?;
    let path = db_path(&app)?;

    Ok(ImportSummary {
        db_path: path.to_string_lossy().to_string(),
        units: table_count(&conn, "units"),
        items: table_count(&conn, "items"),
        convunit: table_count(&conn, "convunit"),
        recp_items: table_count(&conn, "recp_items"),
        inv_units: table_count(&conn, "inv_units"),
        inv_prices: table_count(&conn, "inv_prices"),
        vendors: table_count(&conn, "vendors"),
        recipes: table_count(&conn, "recipes"),
        conv_suggestions: table_count(&conn, "conv_suggestions"),
        conv_suggestions_safe: table_count(&conn, "conv_suggestions_safe"),
        conv_todo: table_count(&conn, "conv_todo"),
        missing_edges: table_count(&conn, "missing_edges"),
        missing_purch_unit: table_count(&conn, "missing_purch_unit"),
        missing_data_report: table_count(&conn, "missing_data_report"),
        invoices: table_count(&conn, "invoices"),
        trans: table_count(&conn, "trans"),
        recp_inv: table_count(&conn, "recp_inv"),
        bids: table_count(&conn, "bids"),
        warnings: Vec::new(),
    })
}

#[tauri::command]
fn import_from_mdf(app: tauri::AppHandle, mdf_path: String) -> Result<ImportSummary, String> {
    let input = resolve_base_path(&mdf_path)?;
    if input.is_dir() {
        if has_required_exports_dir(&input) {
            return import_from_base_dir(&app, &input);
        }
        let exports = input.join("exports");
        if has_required_exports_dir(&exports) {
            return import_from_base_dir(&app, &exports);
        }

        if let Some(mdf) = find_first_mdf_in_dir(&input) {
            let ldf = find_matching_ldf_for_mdf(&mdf);
            return import_from_mdf_pair(&app, &mdf, ldf.as_deref());
        }

        return Err(format!(
            "No CSV exports or MDF file found in {}",
            input.display()
        ));
    }

    if is_mdf_file(&input) {
        if let Ok(exports) = find_exports_dir_for_mdf(&input) {
            if has_required_exports_dir(&exports) {
                return import_from_base_dir(&app, &exports);
            }
        }
        let ldf = find_matching_ldf_for_mdf(&input);
        return import_from_mdf_pair(&app, &input, ldf.as_deref());
    }

    if is_ldf_file(&input) {
        if let Some(mdf) = find_matching_mdf_for_ldf(&input) {
            let ldf = Some(input.as_path());
            return import_from_mdf_pair(&app, &mdf, ldf);
        }
        return Err(format!(
            "Could not locate matching MDF for {}",
            input.display()
        ));
    }

    if let Some(parent) = input.parent() {
        if has_required_exports_dir(parent) {
            return import_from_base_dir(&app, parent);
        }
        let exports = parent.join("exports");
        if has_required_exports_dir(&exports) {
            return import_from_base_dir(&app, &exports);
        }
        if let Some(mdf) = find_first_mdf_in_dir(parent) {
            let ldf = find_matching_ldf_for_mdf(&mdf);
            return import_from_mdf_pair(&app, &mdf, ldf.as_deref());
        }
    }

    Err(
        "Unsupported import path. Provide an exports directory or an MDF/LDF path.".to_string(),
    )
}

#[tauri::command]
fn auto_ingest_invoices(
    inbox_path: String,
    archive_path: String,
    vendor_name: Option<String>,
    parser_profile: Option<String>,
) -> Result<PatchResponse, String> {
    let _ = (vendor_name, parser_profile);
    let inbox = resolve_base_path(&inbox_path)?;
    if !inbox.exists() {
        return Err(format!("Inbox path does not exist: {}", inbox.display()));
    }
    if !inbox.is_dir() {
        return Err(format!(
            "Inbox path is not a directory: {}",
            inbox.display()
        ));
    }

    let invoice_path = find_latest_csv(&inbox, "Invoice.csv")?;
    let trans_path = find_latest_csv(&inbox, "Trans.csv")?;

    if !archive_path.trim().is_empty() {
        let archive = resolve_base_path(&archive_path)?;
        std::fs::create_dir_all(&archive).map_err(|e| e.to_string())?;
        archive_file(&invoice_path, &archive)?;
        archive_file(&trans_path, &archive)?;
    }

    let db_path = fallback_db_path()?;
    let conn = open_db(&db_path)?;
    init_db(&conn)?;
    let mut warnings = Vec::new();
    let (invoice_count, trans_count) =
        import_invoices_and_trans(&conn, &invoice_path, &trans_path, &mut warnings)?;

    let mut message = format!(
        "Imported {} invoices and {} trans lines from {} and {}",
        invoice_count,
        trans_count,
        invoice_path.display(),
        trans_path.display()
    );
    if !warnings.is_empty() {
        message.push_str(&format!(" (warnings: {})", warnings.join("; ")));
    }

    Ok(PatchResponse { ok: true, message })
}

#[tauri::command]
fn auto_ingest_sysco_invoices(
    inbox_path: String,
    archive_path: String,
) -> Result<PatchResponse, String> {
    auto_ingest_invoices(inbox_path, archive_path, None, Some("sysco".to_string()))
}

#[tauri::command]
fn ping_backend() -> Result<PingResponse, String> {
    println!("ping_backend called");
    Ok(PingResponse {
        ok: true,
        message: "pong".to_string(),
    })
}

fn resolve_base_path(input: &str) -> Result<PathBuf, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Base path is empty".to_string());
    }

    let path_str = expand_base_path(trimmed);
    Ok(Path::new(&path_str).to_path_buf())
}

fn is_mdf_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("mdf"))
        .unwrap_or(false)
}

fn find_exports_dir_for_mdf(mdf_path: &Path) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Some(parent) = mdf_path.parent() {
        candidates.push(parent.join("exports"));
        if let Some(grand) = parent.parent() {
            candidates.push(grand.join("exports"));
        }
    }

    for candidate in candidates {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Could not locate exports/ next to MDF at {}",
        mdf_path.display()
    ))
}

fn find_latest_csv(dir: &Path, target_name: &str) -> Result<PathBuf, String> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    let target_lower = target_name.to_ascii_lowercase();
    let target_stem = target_lower.trim_end_matches(".csv");

    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_ascii_lowercase(),
            None => continue,
        };

        let matches = name == target_lower
            || (name.ends_with(".csv") && name.starts_with(target_stem));
        if !matches {
            continue;
        }

        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);

        if best
            .as_ref()
            .map(|(ts, _)| modified > *ts)
            .unwrap_or(true)
        {
            best = Some((modified, path));
        }
    }

    best.map(|(_, path)| path).ok_or_else(|| {
        format!(
            "Could not find {} in {}",
            target_name,
            dir.display()
        )
    })
}

fn archive_file(path: &Path, archive_dir: &Path) -> Result<(), String> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file.csv");
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let dest = archive_dir.join(format!("{}_{}", ts, filename));
    std::fs::copy(path, &dest).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn search_inventory(
    app: tauri::AppHandle,
    query: String,
    limit: u32,
    offset: u32,
) -> Result<InventoryQueryResponse, String> {
    let conn = open_initialized_db(&app)?;
    let trimmed = query.trim();
    let like = format!("%{}%", trimmed);

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    let filtered = if trimmed.is_empty() {
        let filtered = total;
        let mut stmt = conn
            .prepare("SELECT item_id, name, status FROM items ORDER BY name LIMIT ?1 OFFSET ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map((limit, offset), |row| {
                Ok(InventoryItem {
                    item_id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2).ok(),
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            items.push(row.map_err(|e| e.to_string())?);
        }
        filtered
    } else {
        let filtered: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM items WHERE name LIKE ?1 OR CAST(item_id AS TEXT) LIKE ?1",
                [&like],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT item_id, name, status FROM items WHERE name LIKE ?1 OR CAST(item_id AS TEXT) LIKE ?1 ORDER BY name LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map((&like, limit, offset), |row| {
                Ok(InventoryItem {
                    item_id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2).ok(),
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            items.push(row.map_err(|e| e.to_string())?);
        }
        filtered
    };

    Ok(InventoryQueryResponse {
        items,
        total,
        filtered,
    })
}

#[tauri::command]
fn get_inventory_detail(
    app: tauri::AppHandle,
    item_id: i64,
) -> Result<InventoryDetailResponse, String> {
    let conn = open_initialized_db(&app)?;

    let (name, status): (String, Option<i64>) = conn
        .query_row(
            "SELECT name, status FROM items WHERE item_id = ?1",
            [item_id],
            |row| Ok((row.get(0)?, row.get(1).ok())),
        )
        .map_err(|e| e.to_string())?;

    let mut purch_units = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT iu.purch_unit_id, u.sing, iu.is_default, iu.status
             FROM inv_units iu
             LEFT JOIN units u ON u.unit_id = iu.purch_unit_id
             WHERE iu.item_id = ?1
             ORDER BY iu.is_default DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok(InventoryDetailUnit {
                unit_id: row.get(0).ok(),
                unit_name: row.get(1).unwrap_or_else(|_| "-".to_string()),
                is_default: row.get(2).ok(),
                status: row.get(3).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        purch_units.push(row.map_err(|e| e.to_string())?);
    }

    let mut prices = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT ip.vendor_id, v.name, ip.price, ip.pack, ip.status
             FROM inv_prices ip
             LEFT JOIN vendors v ON v.vendor_id = ip.vendor_id
             WHERE ip.item_id = ?1
             ORDER BY ip.vendor_id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok(InventoryDetailPrice {
                vendor_id: row.get(0).ok(),
                vendor_name: row.get(1).unwrap_or_else(|_| "-".to_string()),
                price: row.get(2).ok(),
                pack: row.get(3).unwrap_or_else(|_| "-".to_string()),
                status: row.get(4).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        prices.push(row.map_err(|e| e.to_string())?);
    }

    let mut conversions = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT vendor_id, unit_id1, unit_id2, qty1, qty2
             FROM convunit
             WHERE item_id = ?1
             ORDER BY vendor_id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok(InventoryDetailConversion {
                vendor_id: row.get(0)?,
                unit_id1: row.get(1)?,
                unit_id2: row.get(2)?,
                qty1: row.get(3)?,
                qty2: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        conversions.push(row.map_err(|e| e.to_string())?);
    }

    let mut usage = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT r.recipe_id, r.name, ri.qty, u.sing
             FROM recp_items ri
             LEFT JOIN recipes r ON r.recipe_id = ri.recipe_id
             LEFT JOIN units u ON u.unit_id = ri.unit_id
             WHERE ri.item_id = ?1
             ORDER BY r.name
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok(InventoryDetailUsage {
                recipe_id: row.get(0)?,
                recipe_name: row.get(1).unwrap_or_else(|_| "-".to_string()),
                qty: row.get(2).ok(),
                unit_name: row.get(3).unwrap_or_else(|_| "-".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        usage.push(row.map_err(|e| e.to_string())?);
    }

    let mut missing_edges = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT vendor_id, recipe_unit, purch_unit, hits
             FROM missing_edges
             WHERE item_id = ?1
             ORDER BY hits DESC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok(InventoryDetailMissingEdge {
                vendor_id: row.get(0).unwrap_or(0),
                recipe_unit: row.get(1).unwrap_or_else(|_| "-".to_string()),
                purch_unit: row.get(2).unwrap_or_else(|_| "-".to_string()),
                hits: row.get(3).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        missing_edges.push(row.map_err(|e| e.to_string())?);
    }

    Ok(InventoryDetailResponse {
        item_id,
        name,
        status,
        purch_units,
        prices,
        conversions,
        usage,
        missing_edges,
    })
}

#[tauri::command]
fn search_vendors(
    app: tauri::AppHandle,
    query: String,
    limit: u32,
    offset: u32,
) -> Result<VendorListResponse, String> {
    let conn = open_initialized_db(&app)?;
    let trimmed = query.trim();
    let like = format!("%{}%", trimmed);

    fn normalize_vendor_name(name: &str) -> String {
        name.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace('.', "")
            .replace(',', "")
    }

    let mut raw = Vec::new();
    if trimmed.is_empty() {
        let mut stmt = conn
            .prepare("SELECT vendor_id, name FROM vendors ORDER BY name")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            raw.push(row.map_err(|e| e.to_string())?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT vendor_id, name FROM vendors WHERE name LIKE ?1 OR CAST(vendor_id AS TEXT) LIKE ?1 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([&like], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            raw.push(row.map_err(|e| e.to_string())?);
        }
    }

    let mut map: HashMap<String, VendorListItem> = HashMap::new();
    for (vendor_id, name) in raw {
        let price_items: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT item_id) FROM inv_prices WHERE vendor_id = ?1",
                [vendor_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let key = normalize_vendor_name(&name);
        match map.get(&key) {
            Some(existing) => {
                if price_items > existing.price_items
                    || (price_items == existing.price_items && vendor_id < existing.vendor_id)
                {
                    map.insert(
                        key,
                        VendorListItem {
                            vendor_id,
                            name,
                            price_items,
                        },
                    );
                }
            }
            None => {
                map.insert(
                    key,
                    VendorListItem {
                        vendor_id,
                        name,
                        price_items,
                    },
                );
            }
        }
    }

    let mut vendors: Vec<VendorListItem> = map.into_values().collect();
    vendors.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let total = vendors.len() as i64;
    let start = offset as usize;
    let end = (offset as usize + limit as usize).min(vendors.len());
    let paged = if start < end {
        vendors[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(VendorListResponse {
        vendors: paged,
        total,
        filtered: total,
    })
}

#[tauri::command]
fn get_vendor_detail(
    app: tauri::AppHandle,
    vendor_id: i64,
) -> Result<VendorDetailResponse, String> {
    let conn = open_initialized_db(&app)?;

    let name: String = conn
        .query_row(
            "SELECT name FROM vendors WHERE vendor_id = ?1",
            [vendor_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut price_items = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT ip.item_id, i.name, ip.price, ip.pack
             FROM inv_prices ip
             LEFT JOIN items i ON i.item_id = ip.item_id
             WHERE ip.vendor_id = ?1
             ORDER BY i.name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([vendor_id], |row| {
            Ok(VendorPriceItem {
                item_id: row.get(0)?,
                item_name: row.get(1).unwrap_or_else(|_| "-".to_string()),
                price: row.get(2).ok(),
                pack: row.get(3).unwrap_or_else(|_| "-".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        price_items.push(row.map_err(|e| e.to_string())?);
    }

    let invoice_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE vendor_id = ?1",
            [vendor_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let trans_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM trans WHERE vendor_id = ?1",
            [vendor_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(VendorDetailResponse {
        vendor_id,
        name,
        price_items,
        invoice_count,
        trans_count,
    })
}

#[tauri::command]
fn list_vendors_simple(app: tauri::AppHandle) -> Result<VendorSimpleResponse, String> {
    let conn = open_initialized_db(&app)?;
    let mut raw = Vec::new();
    let mut stmt = conn
        .prepare("SELECT vendor_id, name FROM vendors ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        raw.push(row.map_err(|e| e.to_string())?);
    }

    fn normalize_vendor_name(name: &str) -> String {
        name.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace('.', "")
            .replace(',', "")
    }

    let mut map: HashMap<String, VendorSimple> = HashMap::new();
    for (vendor_id, name) in raw {
        let key = normalize_vendor_name(&name);
        match map.get(&key) {
            Some(existing) => {
                if vendor_id < existing.vendor_id {
                    map.insert(key, VendorSimple { vendor_id, name });
                }
            }
            None => {
                map.insert(key, VendorSimple { vendor_id, name });
            }
        }
    }
    let mut vendors: Vec<VendorSimple> = map.into_values().collect();
    vendors.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(VendorSimpleResponse { vendors })
}

#[tauri::command]
fn list_vendors_all(app: tauri::AppHandle) -> Result<VendorSimpleResponse, String> {
    let conn = open_initialized_db(&app)?;
    let mut vendors = Vec::new();
    let mut stmt = conn
        .prepare("SELECT vendor_id, name FROM vendors ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(VendorSimple {
                vendor_id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        vendors.push(row.map_err(|e| e.to_string())?);
    }
    Ok(VendorSimpleResponse { vendors })
}

#[tauri::command]
fn list_units_simple(app: tauri::AppHandle) -> Result<UnitSimpleResponse, String> {
    let conn = open_initialized_db(&app)?;
    let mut units = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT unit_id, sing
             FROM units
             WHERE sing IS NOT NULL AND TRIM(sing) <> ''
             ORDER BY sing",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(UnitSimple {
                unit_id: row.get(0)?,
                sing: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        units.push(row.map_err(|e| e.to_string())?);
    }
    Ok(UnitSimpleResponse { units })
}

#[tauri::command]
fn list_items_simple(app: tauri::AppHandle) -> Result<ItemSimpleResponse, String> {
    let conn = open_initialized_db(&app)?;
    let mut items = Vec::new();
    let mut stmt = conn
        .prepare("SELECT item_id, name FROM items ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ItemSimple {
                item_id: row.get(0)?,
                name: row.get(1).unwrap_or_else(|_| "-".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    Ok(ItemSimpleResponse { items })
}

#[tauri::command]
fn set_item_purch_unit(
    app: tauri::AppHandle,
    item_id: i64,
    purch_unit_id: i64,
    is_default: bool,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;
    let item_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM items WHERE item_id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if item_exists == 0 {
        return Err(format!("Unknown item_id {}", item_id));
    }
    let unit_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM units WHERE unit_id = ?1",
            [purch_unit_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if unit_exists == 0 {
        return Err(format!("Unknown purch_unit_id {}", purch_unit_id));
    }

    let default_flag = if is_default { 1i64 } else { 0i64 };
    if is_default {
        conn.execute(
            "UPDATE inv_units SET is_default = 0 WHERE item_id = ?1",
            [item_id],
        )
        .map_err(|e| e.to_string())?;
    }

    let updated = conn
        .execute(
            "UPDATE inv_units
             SET is_default = ?3, status = COALESCE(status, 1)
             WHERE item_id = ?1 AND purch_unit_id = ?2",
            (item_id, purch_unit_id, default_flag),
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        conn.execute(
            "INSERT INTO inv_units (item_id, purch_unit_id, is_default, status)
             VALUES (?1, ?2, ?3, ?4)",
            (item_id, purch_unit_id, default_flag, Some(1i64)),
        )
        .map_err(|e| e.to_string())?;
    }
    if is_default {
        conn.execute(
            "UPDATE inv_units SET is_default = 0 WHERE item_id = ?1 AND purch_unit_id <> ?2",
            (item_id, purch_unit_id),
        )
        .map_err(|e| e.to_string())?;
    }
    conn.execute(
        "DELETE FROM missing_purch_unit WHERE item_id = ?1",
        [item_id],
    )
    .ok();

    Ok(PatchResponse {
        ok: true,
        message: format!("Saved purchase unit {} for item {}", purch_unit_id, item_id),
    })
}

#[tauri::command]
fn update_item(
    app: tauri::AppHandle,
    item_id: i64,
    name: String,
    status: Option<i64>,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;
    let final_name = if name.trim().is_empty() {
        "-".to_string()
    } else {
        name.trim().to_string()
    };
    let updated = conn
        .execute(
            "UPDATE items SET name = ?2, status = COALESCE(?3, status) WHERE item_id = ?1",
            (item_id, final_name, status),
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        return Err(format!("Item {} not found", item_id));
    }
    Ok(PatchResponse {
        ok: true,
        message: "Updated item".to_string(),
    })
}

#[tauri::command]
fn update_vendor(
    app: tauri::AppHandle,
    vendor_id: i64,
    name: String,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;
    let final_name = if name.trim().is_empty() {
        "-".to_string()
    } else {
        name.trim().to_string()
    };
    let updated = conn
        .execute(
            "UPDATE vendors SET name = ?2 WHERE vendor_id = ?1",
            (vendor_id, &final_name),
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        conn.execute(
            "INSERT INTO vendors (vendor_id, name) VALUES (?1, ?2)",
            (vendor_id, &final_name),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(PatchResponse {
        ok: true,
        message: "Updated vendor".to_string(),
    })
}

#[tauri::command]
fn update_recipe(
    app: tauri::AppHandle,
    recipe_id: i64,
    name: String,
    instructions: String,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;
    let final_name = if name.trim().is_empty() {
        "-".to_string()
    } else {
        name.trim().to_string()
    };
    let final_instructions = instructions.trim().to_string();

    let updated = conn
        .execute(
            "UPDATE recipes SET name = ?2, instructions = ?3 WHERE recipe_id = ?1",
            (recipe_id, &final_name, &final_instructions),
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        conn.execute(
            "INSERT OR REPLACE INTO recipes (recipe_id, recipe_group_id, name, instructions) VALUES (?1, ?2, ?3, ?4)",
            (recipe_id, recipe_id, &final_name, &final_instructions),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(PatchResponse {
        ok: true,
        message: "Updated recipe".to_string(),
    })
}

#[tauri::command]
fn add_recp_item(
    app: tauri::AppHandle,
    recipe_id: i64,
    recp_item_id: Option<i64>,
    item_id: i64,
    unit_id: Option<i64>,
    qty: Option<f64>,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;

    if let Some(recp_item_id) = recp_item_id {
        if qty.unwrap_or(0.0) <= 0.0 {
            let deleted = conn
                .execute(
                    "DELETE FROM recp_items WHERE recipe_id = ?1 AND recp_item_id = ?2",
                    (recipe_id, recp_item_id),
                )
                .map_err(|e| e.to_string())?;
            if deleted == 0 {
                return Err("Recipe item not found".to_string());
            }
            return Ok(PatchResponse {
                ok: true,
                message: "Deleted recipe item".to_string(),
            });
        }

        let updated = conn
            .execute(
                "UPDATE recp_items SET item_id = ?3, unit_id = ?4, qty = ?5 WHERE recipe_id = ?1 AND recp_item_id = ?2",
                (recipe_id, recp_item_id, item_id, unit_id, qty),
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            conn.execute(
                "INSERT INTO recp_items (recipe_id, recp_item_id, item_id, unit_id, qty) VALUES (?1, ?2, ?3, ?4, ?5)",
                (recipe_id, recp_item_id, item_id, unit_id, qty),
            )
            .map_err(|e| e.to_string())?;
        }
        return Ok(PatchResponse {
            ok: true,
            message: "Updated recipe item".to_string(),
        });
    }

    let next_id: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(recp_item_id), 0) + 1 FROM recp_items WHERE recipe_id = ?1",
            [recipe_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO recp_items (recipe_id, recp_item_id, item_id, unit_id, qty) VALUES (?1, ?2, ?3, ?4, ?5)",
        (recipe_id, next_id, item_id, unit_id, qty),
    )
    .map_err(|e| e.to_string())?;

    Ok(PatchResponse {
        ok: true,
        message: "Added recipe item".to_string(),
    })
}

#[tauri::command]
fn upsert_convunit(
    app: tauri::AppHandle,
    item_id: i64,
    vendor_id: i64,
    unit_id1: i64,
    unit_id2: i64,
    qty1: f64,
    qty2: f64,
    status: Option<i64>,
) -> Result<PatchResponse, String> {
    if qty1 <= 0.0 || qty2 <= 0.0 {
        return Err("Quantity must be greater than 0".to_string());
    }

    let conn = open_initialized_db(&app)?;
    let updated = conn
        .execute(
            "UPDATE convunit
             SET qty1 = ?5, qty2 = ?6, status = COALESCE(?7, status), is_calculated = COALESCE(is_calculated, 1)
             WHERE item_id = ?1 AND vendor_id = ?2 AND unit_id1 = ?3 AND unit_id2 = ?4",
            (item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, status),
        )
        .map_err(|e| e.to_string())?;

    if updated == 0 {
        conn.execute(
            "INSERT INTO convunit (item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, status, is_calculated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, status, Some(1i64)),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(PatchResponse {
        ok: true,
        message: "Saved conversion".to_string(),
    })
}

#[tauri::command]
fn update_invoice(
    app: tauri::AppHandle,
    invoice_id: i64,
    invoice_no: String,
    invoice_date: String,
    vendor_id: Option<i64>,
    freight: Option<f64>,
    total: Option<f64>,
    status: Option<i64>,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;

    let updated = conn
        .execute(
            "UPDATE invoices
             SET invoice_no = ?2, invoice_date = ?3, vendor_id = ?4, freight = ?5, total = ?6, status = COALESCE(?7, status)
             WHERE invoice_id = ?1",
            (
                invoice_id,
                invoice_no.trim().to_string(),
                invoice_date.trim().to_string(),
                vendor_id,
                freight,
                total,
                status,
            ),
        )
        .map_err(|e| e.to_string())?;

    if updated == 0 {
        return Err(format!("Invoice {} not found", invoice_id));
    }

    Ok(PatchResponse {
        ok: true,
        message: "Updated invoice".to_string(),
    })
}

#[tauri::command]
fn update_trans_line(
    app: tauri::AppHandle,
    trans_id: i64,
    qty: Option<f64>,
    unit_id: Option<i64>,
    price: Option<f64>,
    ext_cost: Option<f64>,
    status: Option<i64>,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;

    let updated = conn
        .execute(
            "UPDATE trans
             SET qty = COALESCE(?2, qty),
                 unit_id = COALESCE(?3, unit_id),
                 price = COALESCE(?4, price),
                 ext_cost = COALESCE(?5, ext_cost),
                 status = COALESCE(?6, status)
             WHERE trans_id = ?1",
            (trans_id, qty, unit_id, price, ext_cost, status),
        )
        .map_err(|e| e.to_string())?;

    if updated == 0 {
        return Err(format!("Transaction {} not found", trans_id));
    }

    Ok(PatchResponse {
        ok: true,
        message: "Updated transaction line".to_string(),
    })
}

#[tauri::command]
fn recalculate_reports(app: tauri::AppHandle) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;

    conn.execute(
        "DELETE FROM missing_purch_unit
         WHERE item_id IN (
           SELECT DISTINCT item_id
           FROM inv_units
           WHERE purch_unit_id IS NOT NULL
         )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM missing_data_report
         WHERE recipe_id NOT IN (SELECT recipe_id FROM recipes)",
        [],
    )
    .map_err(|e| e.to_string())?;

    let overview = get_conversion_overview(app)?;
    Ok(PatchResponse {
        ok: true,
        message: format!(
            "Recalculated reports. suggestions={}, todo={}, missing_edges={}, missing_purch={}, missing_data={}",
            overview.suggestions,
            overview.todo,
            overview.missing_edges,
            overview.missing_purch,
            overview.missing_data
        ),
    })
}

#[tauri::command]
fn revert_db(app: tauri::AppHandle) -> Result<PatchResponse, String> {
    let path = db_path(&app)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let backup = path.with_file_name(format!("4chef.db.bak.{}", ts));

    if path.exists() {
        std::fs::copy(&path, &backup).map_err(|e| e.to_string())?;
    }

    let mut conn = open_db(&path)?;
    init_db(&conn)?;
    with_tx(&mut conn, |tx| {
        clear_tables(tx)?;
        Ok(())
    })?;

    Ok(PatchResponse {
        ok: true,
        message: format!("Database reverted. Backup saved to {}", backup.display()),
    })
}

#[tauri::command]
fn upsert_manual_price(
    app: tauri::AppHandle,
    item_id: i64,
    vendor_id: i64,
    price: f64,
    pack: String,
) -> Result<PatchResponse, String> {
    if price <= 0.0 {
        return Err("Price must be greater than 0".to_string());
    }
    let conn = open_initialized_db(&app)?;

    let item_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM items WHERE item_id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if item_exists == 0 {
        return Err(format!("Unknown item_id {}", item_id));
    }
    let vendor_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM vendors WHERE vendor_id = ?1",
            [vendor_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if vendor_exists == 0 {
        return Err(format!("Unknown vendor_id {}", vendor_id));
    }

    let pack = if pack.trim().is_empty() {
        "-".to_string()
    } else {
        pack.trim().to_string()
    };

    let updated = conn
        .execute(
            "UPDATE inv_prices
             SET price = ?3, pack = ?4, status = ?5
             WHERE item_id = ?1 AND vendor_id = ?2",
            (item_id, vendor_id, price, &pack, Some(1i64)),
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        conn.execute(
            "INSERT INTO inv_prices (item_id, vendor_id, price, pack, status)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (item_id, vendor_id, price, &pack, Some(1i64)),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(PatchResponse {
        ok: true,
        message: format!(
            "Saved price {} for item {} vendor {}",
            price, item_id, vendor_id
        ),
    })
}

#[tauri::command]
fn merge_vendor(
    app: tauri::AppHandle,
    source_vendor_id: i64,
    target_vendor_id: i64,
) -> Result<PatchResponse, String> {
    if source_vendor_id == target_vendor_id {
        return Err("Source and target vendor IDs must be different".to_string());
    }

    let mut conn = open_initialized_db(&app)?;
    let (
        target_name,
        source_name,
        moved_prices,
        merged_prices,
        moved_conversions,
        merged_conversions,
        moved_rows,
    ) = with_tx(&mut conn, |tx| {
        let source_name: String = tx
            .query_row(
                "SELECT name FROM vendors WHERE vendor_id = ?1",
                [source_vendor_id],
                |row| row.get(0),
            )
            .map_err(|_| format!("Source vendor {} not found", source_vendor_id))?;
        let target_name: String = tx
            .query_row(
                "SELECT name FROM vendors WHERE vendor_id = ?1",
                [target_vendor_id],
                |row| row.get(0),
            )
            .map_err(|_| format!("Target vendor {} not found", target_vendor_id))?;

        let mut moved_prices = 0usize;
        let mut merged_prices = 0usize;
        let mut moved_conversions = 0usize;
        let mut merged_conversions = 0usize;
        let mut moved_rows = 0usize;

        let mut stmt = tx
            .prepare(
                "SELECT rowid, item_id, price, pack, status
                     FROM inv_prices
                     WHERE vendor_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([source_vendor_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<f64>>(2).ok().flatten(),
                    row.get::<_, String>(3).unwrap_or_else(|_| "".to_string()),
                    row.get::<_, Option<i64>>(4).ok().flatten(),
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut source_prices = Vec::new();
        for row in rows {
            source_prices.push(row.map_err(|e| e.to_string())?);
        }

        for (row_id, item_id, price, pack, status) in source_prices {
            let target_row: Option<(i64, Option<f64>, String, Option<i64>)> = tx
                .query_row(
                    "SELECT rowid, price, pack, status
                         FROM inv_prices
                         WHERE item_id = ?1 AND vendor_id = ?2
                         ORDER BY rowid
                         LIMIT 1",
                    (item_id, target_vendor_id),
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, Option<f64>>(1).ok().flatten(),
                            row.get::<_, String>(2).unwrap_or_else(|_| "".to_string()),
                            row.get::<_, Option<i64>>(3).ok().flatten(),
                        ))
                    },
                )
                .optional()
                .map_err(|e| e.to_string())?;

            if let Some((target_row_id, target_price, target_pack, target_status)) = target_row {
                let merged_price = if target_price.unwrap_or(0.0) > 0.0 {
                    target_price
                } else {
                    price
                };
                let merged_pack = if target_pack.trim().is_empty() {
                    pack.trim().to_string()
                } else {
                    target_pack
                };
                let merged_status = target_status.or(status);

                tx.execute(
                    "UPDATE inv_prices SET price = ?1, pack = ?2, status = ?3 WHERE rowid = ?4",
                    (merged_price, merged_pack, merged_status, target_row_id),
                )
                .map_err(|e| e.to_string())?;
                tx.execute("DELETE FROM inv_prices WHERE rowid = ?1", [row_id])
                    .map_err(|e| e.to_string())?;
                merged_prices += 1;
            } else {
                tx.execute(
                    "UPDATE inv_prices SET vendor_id = ?1 WHERE rowid = ?2",
                    (target_vendor_id, row_id),
                )
                .map_err(|e| e.to_string())?;
                moved_prices += 1;
            }
        }

        let mut stmt = tx
            .prepare(
                "SELECT rowid, item_id, unit_id1, unit_id2, qty1, qty2, status, is_calculated
                     FROM convunit
                     WHERE vendor_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([source_vendor_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, f64>(5)?,
                    row.get::<_, Option<i64>>(6).ok().flatten(),
                    row.get::<_, Option<i64>>(7).ok().flatten(),
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut source_conversions = Vec::new();
        for row in rows {
            source_conversions.push(row.map_err(|e| e.to_string())?);
        }

        for (row_id, item_id, unit_id1, unit_id2, qty1, qty2, status, is_calculated) in
            source_conversions
        {
            let target_row: Option<(i64, f64, f64, Option<i64>, Option<i64>)> = tx
                .query_row(
                    "SELECT rowid, qty1, qty2, status, is_calculated
                         FROM convunit
                         WHERE item_id = ?1 AND vendor_id = ?2 AND unit_id1 = ?3 AND unit_id2 = ?4
                         ORDER BY rowid
                         LIMIT 1",
                    (item_id, target_vendor_id, unit_id1, unit_id2),
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, f64>(1)?,
                            row.get::<_, f64>(2)?,
                            row.get::<_, Option<i64>>(3).ok().flatten(),
                            row.get::<_, Option<i64>>(4).ok().flatten(),
                        ))
                    },
                )
                .optional()
                .map_err(|e| e.to_string())?;

            if let Some((target_row_id, target_qty1, target_qty2, target_status, target_calc)) =
                target_row
            {
                let merged_qty1 = if target_qty1 > 0.0 { target_qty1 } else { qty1 };
                let merged_qty2 = if target_qty2 > 0.0 { target_qty2 } else { qty2 };
                let merged_status = target_status.or(status);
                let merged_calc = target_calc.or(is_calculated);

                tx.execute(
                    "UPDATE convunit
                         SET qty1 = ?1, qty2 = ?2, status = ?3, is_calculated = ?4
                         WHERE rowid = ?5",
                    (
                        merged_qty1,
                        merged_qty2,
                        merged_status,
                        merged_calc,
                        target_row_id,
                    ),
                )
                .map_err(|e| e.to_string())?;
                tx.execute("DELETE FROM convunit WHERE rowid = ?1", [row_id])
                    .map_err(|e| e.to_string())?;
                merged_conversions += 1;
            } else {
                tx.execute(
                    "UPDATE convunit SET vendor_id = ?1 WHERE rowid = ?2",
                    (target_vendor_id, row_id),
                )
                .map_err(|e| e.to_string())?;
                moved_conversions += 1;
            }
        }

        for table in [
            "invoices",
            "trans",
            "conv_suggestions",
            "conv_suggestions_safe",
            "conv_todo",
            "missing_edges",
        ] {
            moved_rows += tx
                .execute(
                    &format!("UPDATE {} SET vendor_id = ?1 WHERE vendor_id = ?2", table),
                    (target_vendor_id, source_vendor_id),
                )
                .map_err(|e| e.to_string())?;
        }

        let deleted = tx
            .execute(
                "DELETE FROM vendors WHERE vendor_id = ?1",
                [source_vendor_id],
            )
            .map_err(|e| e.to_string())?;
        if deleted == 0 {
            return Err(format!("Source vendor {} not found", source_vendor_id));
        }

        Ok((
            target_name,
            source_name,
            moved_prices,
            merged_prices,
            moved_conversions,
            merged_conversions,
            moved_rows,
        ))
    })?;

    Ok(PatchResponse {
        ok: true,
        message: format!(
            "Merged '{}' ({}) into '{}' ({}). Prices moved/merged: {}/{}. Conversions moved/merged: {}/{}. Related rows updated: {}",
            source_name,
            source_vendor_id,
            target_name,
            target_vendor_id,
            moved_prices,
            merged_prices,
            moved_conversions,
            merged_conversions,
            moved_rows
        ),
    })
}

#[tauri::command]
fn search_recipes(
    app: tauri::AppHandle,
    query: String,
    limit: u32,
    offset: u32,
) -> Result<RecipeListResponse, String> {
    let conn = open_initialized_db(&app)?;
    let trimmed = query.trim();
    let like = format!("%{}%", trimmed);

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM recipes", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut recipes = Vec::new();
    let filtered = if trimmed.is_empty() {
        let filtered = total;
        let mut stmt = conn
            .prepare("SELECT recipe_id, name FROM recipes ORDER BY name LIMIT ?1 OFFSET ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map((limit, offset), |row| {
                let recipe_id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((recipe_id, name))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (recipe_id, name) = row.map_err(|e| e.to_string())?;
            let item_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM recp_items WHERE recipe_id = ?1",
                    [recipe_id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            recipes.push(RecipeListItem {
                recipe_id,
                name,
                item_count,
            });
        }
        filtered
    } else {
        let filtered: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM recipes WHERE name LIKE ?1 OR CAST(recipe_id AS TEXT) LIKE ?1",
                [&like],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT recipe_id, name FROM recipes WHERE name LIKE ?1 OR CAST(recipe_id AS TEXT) LIKE ?1 ORDER BY name LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map((&like, limit, offset), |row| {
                let recipe_id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((recipe_id, name))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (recipe_id, name) = row.map_err(|e| e.to_string())?;
            let item_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM recp_items WHERE recipe_id = ?1",
                    [recipe_id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            recipes.push(RecipeListItem {
                recipe_id,
                name,
                item_count,
            });
        }
        filtered
    };

    Ok(RecipeListResponse {
        recipes,
        total,
        filtered,
    })
}

fn build_conversion_edges(
    conn: &rusqlite::Connection,
    item_id: i64,
    vendor_id: Option<i64>,
) -> Result<Vec<(i64, i64, f64, f64)>, String> {
    let mut edges = Vec::new();
    if let Some(vendor_id) = vendor_id {
        let mut stmt = conn
            .prepare(
                "SELECT unit_id1, unit_id2, qty1, qty2
                 FROM convunit
                 WHERE item_id = ?1 AND (vendor_id = ?2 OR vendor_id = 0)",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map((item_id, vendor_id), |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            edges.push(row.map_err(|e| e.to_string())?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT unit_id1, unit_id2, qty1, qty2
                 FROM convunit
                 WHERE item_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([item_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            edges.push(row.map_err(|e| e.to_string())?);
        }
    }
    Ok(edges)
}

fn bfs_conversion_factor(
    edges: &[(i64, i64, f64, f64)],
    from_unit: i64,
    to_unit: i64,
    max_hops: usize,
) -> Option<(f64, usize)> {
    use std::collections::{HashMap, VecDeque};

    if from_unit == to_unit {
        return Some((1.0, 0));
    }

    let mut graph: HashMap<i64, Vec<(i64, f64)>> = HashMap::new();
    for (u1, u2, q1, q2) in edges.iter() {
        if *q1 > 0.0 && *q2 > 0.0 {
            graph.entry(*u1).or_default().push((*u2, q2 / q1));
            graph.entry(*u2).or_default().push((*u1, q1 / q2));
        }
    }

    let mut queue = VecDeque::new();
    let mut visited: HashMap<i64, usize> = HashMap::new();
    queue.push_back((from_unit, 1.0f64, 0usize));
    visited.insert(from_unit, 0);

    while let Some((unit, factor, hops)) = queue.pop_front() {
        if hops >= max_hops {
            continue;
        }
        if let Some(neighbors) = graph.get(&unit) {
            for (next, edge_factor) in neighbors {
                let next_hops = hops + 1;
                if visited.get(next).map(|h| *h <= next_hops).unwrap_or(false) {
                    continue;
                }
                let new_factor = factor * edge_factor;
                if *next == to_unit {
                    return Some((new_factor, next_hops));
                }
                visited.insert(*next, next_hops);
                queue.push_back((*next, new_factor, next_hops));
            }
        }
    }

    None
}

#[tauri::command]
fn get_recipe_detail(
    app: tauri::AppHandle,
    recipe_id: i64,
) -> Result<RecipeDetailResponse, String> {
    let conn = open_initialized_db(&app)?;

    let (name, instructions): (String, Option<String>) = conn
        .query_row(
            "SELECT name, instructions FROM recipes WHERE recipe_id = ?1",
            [recipe_id],
            |row| Ok((row.get(0)?, row.get(1).ok())),
        )
        .map_err(|e| e.to_string())?;
    let instructions = instructions.unwrap_or_default();

    let item_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM recp_items WHERE recipe_id = ?1",
            [recipe_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT ri.recp_item_id, ri.item_id, i.name, ri.unit_id, u.sing, ri.qty
             FROM recp_items ri
             LEFT JOIN items i ON i.item_id = ri.item_id
             LEFT JOIN units u ON u.unit_id = ri.unit_id
             WHERE ri.recipe_id = ?1
             ORDER BY i.name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([recipe_id], |row| {
            let recp_item_id: i64 = row.get(0)?;
            let item_id: i64 = row.get(1)?;
            let item_name: Option<String> = row.get(2).ok();
            let unit_id: Option<i64> = row.get(3).ok();
            let unit_name: Option<String> = row.get(4).ok();
            let qty: Option<f64> = row.get(5).ok();
            Ok((recp_item_id, item_id, item_name, unit_id, unit_name, qty))
        })
        .map_err(|e| e.to_string())?;

    let mut ingredients = Vec::new();
    let mut total_cost = 0.0f64;
    let mut missing_costs = 0i64;

    for row in rows {
        let (recp_item_id, item_id, item_name, unit_id, unit_name, qty) = row.map_err(|e| e.to_string())?;

        let item_name = item_name.unwrap_or_else(|| "(unknown item)".to_string());
        let unit_name = unit_name.unwrap_or_else(|| "-".to_string());

        let purch_unit_id: Option<i64> = conn
            .query_row(
                "SELECT purch_unit_id FROM inv_units WHERE item_id = ?1 AND is_default = 1 LIMIT 1",
                [item_id],
                |row| row.get(0),
            )
            .ok();

        let purch_unit_id = purch_unit_id.or_else(|| {
            conn.query_row(
                "SELECT purch_unit_id FROM inv_units WHERE item_id = ?1 LIMIT 1",
                [item_id],
                |row| row.get(0),
            )
            .ok()
        });

        let purch_unit_name = if let Some(pid) = purch_unit_id {
            conn.query_row("SELECT sing FROM units WHERE unit_id = ?1", [pid], |row| {
                row.get(0)
            })
            .unwrap_or_else(|_| "-".to_string())
        } else {
            "-".to_string()
        };

        let price_vendor: Option<(f64, i64)> = conn
            .query_row(
                "SELECT price, vendor_id FROM inv_prices WHERE item_id = ?1 ORDER BY vendor_id LIMIT 1",
                [item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok()
            .or_else(|| {
                conn.query_row(
                    "SELECT price, vendor_id FROM trans WHERE item_id = ?1 AND price IS NOT NULL AND price > 0 ORDER BY trans_date DESC, trans_id DESC LIMIT 1",
                    [item_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok()
            });
        let price = price_vendor.map(|v| v.0);
        let price_vendor_id = price_vendor.map(|v| v.1);

        let mut factor: Option<f64> = None;
        let mut hops = 0usize;
        if let (Some(recipe_unit_id), Some(purch_unit_id)) = (unit_id, purch_unit_id) {
            if recipe_unit_id == purch_unit_id {
                factor = Some(1.0);
            } else {
                let edges = build_conversion_edges(&conn, item_id, price_vendor_id)?;
                if let Some((found_factor, found_hops)) =
                    bfs_conversion_factor(&edges, recipe_unit_id, purch_unit_id, 6)
                {
                    factor = Some(found_factor);
                    hops = found_hops;
                }
            }
        }

        let qty_valid = qty.unwrap_or(0.0) > 0.0;

        let (cost_status, extended_cost) = if !qty_valid {
            ("Missing qty".to_string(), None)
        } else if purch_unit_id.is_none() {
            ("Missing purch unit".to_string(), None)
        } else if price.is_none() {
            ("Missing price".to_string(), None)
        } else if factor.is_none() {
            ("Needs conversion".to_string(), None)
        } else {
            let cost = qty.unwrap_or(0.0) * factor.unwrap_or(1.0) * price.unwrap_or(0.0);
            if hops > 0 {
                (format!("OK ({} hops)", hops), Some(cost))
            } else {
                ("OK".to_string(), Some(cost))
            }
        };

        if let Some(cost) = extended_cost {
            total_cost += cost;
        } else {
            missing_costs += 1;
        }

        ingredients.push(RecipeIngredient {
            recp_item_id,
            item_id,
            item_name,
            unit_id,
            unit_name,
            qty,
            purch_unit_id,
            purch_unit_name,
            price,
            extended_cost,
            cost_status,
        });
    }

    Ok(RecipeDetailResponse {
        recipe_id,
        name,
        item_count,
        total_cost,
        missing_costs,
        instructions,
        ingredients,
    })
}

#[tauri::command]
fn get_conversion_overview(app: tauri::AppHandle) -> Result<ConversionOverview, String> {
    let conn = open_initialized_db(&app)?;
    let suggestions: i64 = conn
        .query_row("SELECT COUNT(*) FROM conv_suggestions", [], |r| r.get(0))
        .unwrap_or(0);
    let suggestions_safe: i64 = conn
        .query_row("SELECT COUNT(*) FROM conv_suggestions_safe", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let todo: i64 = conn
        .query_row("SELECT COUNT(*) FROM conv_todo", [], |r| r.get(0))
        .unwrap_or(0);
    let missing_edges: i64 = conn
        .query_row("SELECT COUNT(*) FROM missing_edges", [], |r| r.get(0))
        .unwrap_or(0);
    let missing_purch: i64 = conn
        .query_row("SELECT COUNT(*) FROM missing_purch_unit", [], |r| r.get(0))
        .unwrap_or(0);
    let missing_data: i64 = conn
        .query_row("SELECT COUNT(*) FROM missing_data_report", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(ConversionOverview {
        suggestions,
        suggestions_safe,
        todo,
        missing_edges,
        missing_purch,
        missing_data,
    })
}

#[tauri::command]
fn list_conv_suggestions(
    app: tauri::AppHandle,
    table: String,
    limit: u32,
    offset: u32,
) -> Result<ConversionSuggestionResponse, String> {
    let conn = open_initialized_db(&app)?;
    let table = match table.as_str() {
        "conv_suggestions" => "conv_suggestions",
        "conv_suggestions_safe" => "conv_suggestions_safe",
        _ => return Err("Invalid suggestions table".to_string()),
    };
    let total: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
        .unwrap_or(0);

    let mut rows = Vec::new();
    let sql = format!(
        "SELECT item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, recipe_unit, purch_unit, hits, derived_from, hops, path
         FROM {}
         ORDER BY hits DESC
         LIMIT ?1 OFFSET ?2",
        table
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map((limit, offset), |row| {
            Ok(ConversionSuggestionRow {
                item_id: row.get(0)?,
                vendor_id: row.get(1)?,
                unit_id1: row.get(2)?,
                unit_id2: row.get(3)?,
                qty1: row.get(4)?,
                qty2: row.get(5)?,
                recipe_unit: row.get(6).unwrap_or_else(|_| "-".to_string()),
                purch_unit: row.get(7).unwrap_or_else(|_| "-".to_string()),
                hits: row.get(8).ok(),
                derived_from: row.get(9).unwrap_or_else(|_| "-".to_string()),
                hops: row.get(10).ok(),
                path: row.get(11).unwrap_or_else(|_| "-".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in iter {
        rows.push(row.map_err(|e| e.to_string())?);
    }
    Ok(ConversionSuggestionResponse { rows, total })
}

#[tauri::command]
fn list_conv_todo(
    app: tauri::AppHandle,
    limit: u32,
    offset: u32,
) -> Result<ConversionTodoResponse, String> {
    let conn = open_initialized_db(&app)?;
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM conv_todo", [], |r| r.get(0))
        .unwrap_or(0);
    let mut rows = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT item_id, vendor_id, recipe_unit_id, purch_unit_id, recipe_unit, purch_unit, hits, needed
             FROM conv_todo
             ORDER BY hits DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map((limit, offset), |row| {
            Ok(ConversionTodoRow {
                item_id: row.get(0)?,
                vendor_id: row.get(1)?,
                recipe_unit_id: row.get(2)?,
                purch_unit_id: row.get(3)?,
                recipe_unit: row.get(4).unwrap_or_else(|_| "-".to_string()),
                purch_unit: row.get(5).unwrap_or_else(|_| "-".to_string()),
                hits: row.get(6).ok(),
                needed: row.get(7).unwrap_or_else(|_| "-".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in iter {
        rows.push(row.map_err(|e| e.to_string())?);
    }
    Ok(ConversionTodoResponse { rows, total })
}

#[tauri::command]
fn list_missing_edges(
    app: tauri::AppHandle,
    limit: u32,
    offset: u32,
) -> Result<MissingEdgeResponse, String> {
    let conn = open_initialized_db(&app)?;
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM missing_edges", [], |r| r.get(0))
        .unwrap_or(0);
    let mut rows = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT item_id, item_name, vendor_id, recipe_unit_id, recipe_unit, purch_unit_id, purch_unit, hits
             FROM missing_edges
             ORDER BY hits DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map((limit, offset), |row| {
            Ok(MissingEdgeRow {
                item_id: row.get(0)?,
                item_name: row.get(1).unwrap_or_else(|_| "-".to_string()),
                vendor_id: row.get(2).unwrap_or(0),
                recipe_unit_id: row.get(3)?,
                recipe_unit: row.get(4).unwrap_or_else(|_| "-".to_string()),
                purch_unit_id: row.get(5)?,
                purch_unit: row.get(6).unwrap_or_else(|_| "-".to_string()),
                hits: row.get(7).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in iter {
        rows.push(row.map_err(|e| e.to_string())?);
    }
    Ok(MissingEdgeResponse { rows, total })
}

#[tauri::command]
fn list_missing_purch_unit(
    app: tauri::AppHandle,
    limit: u32,
    offset: u32,
) -> Result<MissingPurchResponse, String> {
    let conn = open_initialized_db(&app)?;
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM missing_purch_unit", [], |r| r.get(0))
        .unwrap_or(0);
    let mut rows = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT item_id, item_name, usage_count
             FROM missing_purch_unit
             ORDER BY usage_count DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map((limit, offset), |row| {
            Ok(MissingPurchRow {
                item_id: row.get(0)?,
                item_name: row.get(1).unwrap_or_else(|_| "-".to_string()),
                usage_count: row.get(2).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in iter {
        rows.push(row.map_err(|e| e.to_string())?);
    }
    Ok(MissingPurchResponse { rows, total })
}

#[tauri::command]
fn list_missing_data_report(
    app: tauri::AppHandle,
    limit: u32,
    offset: u32,
) -> Result<MissingDataResponse, String> {
    let conn = open_initialized_db(&app)?;
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM missing_data_report", [], |r| r.get(0))
        .unwrap_or(0);
    let mut rows = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT recipe_id, recipe_name, missing_a, missing_b, missing_c
             FROM missing_data_report
             ORDER BY missing_a DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map((limit, offset), |row| {
            Ok(MissingDataRow {
                recipe_id: row.get(0)?,
                recipe_name: row.get(1).unwrap_or_else(|_| "-".to_string()),
                missing_a: row.get(2).ok(),
                missing_b: row.get(3).ok(),
                missing_c: row.get(4).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in iter {
        rows.push(row.map_err(|e| e.to_string())?);
    }
    Ok(MissingDataResponse { rows, total })
}

#[tauri::command]
fn list_invoices(
    app: tauri::AppHandle,
    query: String,
    vendor_id: Option<i64>,
    date_from: String,
    date_to: String,
    limit: u32,
    offset: u32,
) -> Result<InvoiceListResponse, String> {
    let conn = open_initialized_db(&app)?;
    let trimmed = query.trim();
    let like = format!("%{}%", trimmed);

    let mut where_clauses = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    if !trimmed.is_empty() {
        where_clauses.push("i.invoice_no LIKE ?1 OR CAST(i.invoice_id AS TEXT) LIKE ?1 OR CAST(i.vendor_id AS TEXT) LIKE ?1");
        params.push(rusqlite::types::Value::from(like.clone()));
    }
    if let Some(vendor_id) = vendor_id {
        where_clauses.push("i.vendor_id = ?");
        params.push(rusqlite::types::Value::from(vendor_id));
    }
    if !date_from.trim().is_empty() {
        where_clauses.push("i.invoice_date >= ?");
        params.push(rusqlite::types::Value::from(date_from.trim().to_string()));
    }
    if !date_to.trim().is_empty() {
        where_clauses.push("i.invoice_date <= ?");
        params.push(rusqlite::types::Value::from(date_to.trim().to_string()));
    }

    let where_sql = if where_clauses.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let total_sql = format!("SELECT COUNT(*) FROM invoices i {}", where_sql);
    let total: i64 = conn
        .query_row(
            &total_sql,
            rusqlite::params_from_iter(params.clone()),
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut invoices = Vec::new();
    let mut params_for_query = params.clone();
    params_for_query.push(rusqlite::types::Value::from(limit as i64));
    params_for_query.push(rusqlite::types::Value::from(offset as i64));
    let list_sql = format!(
        "SELECT i.invoice_id, i.invoice_no, i.vendor_id, v.name, i.invoice_date, i.total
         FROM invoices i
         LEFT JOIN vendors v ON v.vendor_id = i.vendor_id
         {}
         ORDER BY i.invoice_date DESC
         LIMIT ? OFFSET ?",
        where_sql
    );
    let mut stmt = conn.prepare(&list_sql).map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map(rusqlite::params_from_iter(params_for_query), |row| {
            Ok(InvoiceListItem {
                invoice_id: row.get(0)?,
                invoice_no: row.get(1).unwrap_or_else(|_| "-".to_string()),
                vendor_id: row.get(2).ok(),
                vendor_name: row.get(3).unwrap_or_else(|_| "-".to_string()),
                invoice_date: row.get(4).unwrap_or_else(|_| "-".to_string()),
                total: row.get(5).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in iter {
        invoices.push(row.map_err(|e| e.to_string())?);
    }

    Ok(InvoiceListResponse {
        invoices,
        total,
        filtered: total,
    })
}

#[tauri::command]
fn get_invoice_detail(
    app: tauri::AppHandle,
    invoice_id: i64,
) -> Result<InvoiceDetailResponse, String> {
    let conn = open_initialized_db(&app)?;
    let invoice = conn
        .query_row(
            "SELECT i.invoice_id, i.invoice_no, i.vendor_id, v.name, i.invoice_date, i.total, i.freight
             FROM invoices i
             LEFT JOIN vendors v ON v.vendor_id = i.vendor_id
             WHERE i.invoice_id = ?1",
            [invoice_id],
            |row| {
                Ok((
                    InvoiceListItem {
                        invoice_id: row.get(0)?,
                        invoice_no: row.get(1).unwrap_or_else(|_| "-".to_string()),
                        vendor_id: row.get(2).ok(),
                        vendor_name: row.get(3).unwrap_or_else(|_| "-".to_string()),
                        invoice_date: row.get(4).unwrap_or_else(|_| "-".to_string()),
                        total: row.get(5).ok(),
                    },
                    row.get::<_, Option<f64>>(6).ok().flatten(),
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let mut lines = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT t.trans_id, t.item_id, i.name, t.qty, u.sing, t.price, t.ext_cost
             FROM trans t
             LEFT JOIN items i ON i.item_id = t.item_id
             LEFT JOIN units u ON u.unit_id = t.unit_id
             WHERE t.invoice_id = ?1
             ORDER BY i.name",
        )
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map([invoice_id], |row| {
            Ok(InvoiceLineItem {
                trans_id: row.get(0).ok(),
                item_id: row.get(1).ok(),
                item_name: row.get(2).unwrap_or_else(|_| "-".to_string()),
                qty: row.get(3).ok(),
                unit_name: row.get(4).unwrap_or_else(|_| "-".to_string()),
                price: row.get(5).ok(),
                ext_cost: row.get(6).ok(),
            })
        })
        .map_err(|e| e.to_string())?;
    for row in iter {
        lines.push(row.map_err(|e| e.to_string())?);
    }

    Ok(InvoiceDetailResponse {
        invoice: invoice.0,
        freight: invoice.1,
        lines,
    })
}

#[tauri::command]
fn export_invoice_lines_csv(
    app: tauri::AppHandle,
    query: String,
    vendor_id: Option<i64>,
    date_from: String,
    date_to: String,
    output_path: Option<String>,
) -> Result<ExportCsvResponse, String> {
    let conn = open_initialized_db(&app)?;
    let trimmed = query.trim();
    let like = format!("%{}%", trimmed);

    let mut where_clauses = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if !trimmed.is_empty() {
        where_clauses.push("(i.invoice_no LIKE ? OR CAST(i.invoice_id AS TEXT) LIKE ?)");
        params.push(rusqlite::types::Value::from(like.clone()));
        params.push(rusqlite::types::Value::from(like));
    }
    if let Some(vendor_id) = vendor_id {
        where_clauses.push("i.vendor_id = ?");
        params.push(rusqlite::types::Value::from(vendor_id));
    }
    if !date_from.trim().is_empty() {
        where_clauses.push("i.invoice_date >= ?");
        params.push(rusqlite::types::Value::from(date_from.trim().to_string()));
    }
    if !date_to.trim().is_empty() {
        where_clauses.push("i.invoice_date <= ?");
        params.push(rusqlite::types::Value::from(date_to.trim().to_string()));
    }

    let where_sql = if where_clauses.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let sql = format!(
        "SELECT i.invoice_id, i.invoice_no, i.invoice_date, i.vendor_id, v.name,
                t.trans_id, t.item_id, it.name, t.qty, u.sing, t.price, t.ext_cost
         FROM invoices i
         LEFT JOIN vendors v ON v.vendor_id = i.vendor_id
         LEFT JOIN trans t ON t.invoice_id = i.invoice_id
         LEFT JOIN items it ON it.item_id = t.item_id
         LEFT JOIN units u ON u.unit_id = t.unit_id
         {}
         ORDER BY i.invoice_date DESC, i.invoice_id DESC",
        where_sql
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1).unwrap_or_else(|_| "-".to_string()),
                row.get::<_, String>(2).unwrap_or_else(|_| "-".to_string()),
                row.get::<_, Option<i64>>(3).unwrap_or(None),
                row.get::<_, String>(4).unwrap_or_else(|_| "-".to_string()),
                row.get::<_, Option<i64>>(5).unwrap_or(None),
                row.get::<_, Option<i64>>(6).unwrap_or(None),
                row.get::<_, String>(7).unwrap_or_else(|_| "-".to_string()),
                row.get::<_, Option<f64>>(8).unwrap_or(None),
                row.get::<_, String>(9).unwrap_or_else(|_| "-".to_string()),
                row.get::<_, Option<f64>>(10).unwrap_or(None),
                row.get::<_, Option<f64>>(11).unwrap_or(None),
            ))
        })
        .map_err(|e| e.to_string())?;

    let path = if let Some(path) = output_path {
        path
    } else {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let data_root = db_path(&app)
            .ok()
            .and_then(|db| db.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(fallback_data_root);
        let reports_dir = data_root.join("reports");
        std::fs::create_dir_all(&reports_dir).map_err(|e| e.to_string())?;
        reports_dir
            .join(format!("invoice_lines_{}.csv", ts))
            .to_string_lossy()
            .to_string()
    };

    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| e.to_string())?;

    writeln!(
        file,
        "InvoiceID,InvoiceNo,InvoiceDate,VendorID,VendorName,TransID,ItemID,ItemName,Qty,Unit,Price,ExtCost"
    )
    .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for row in rows {
        let (
            invoice_id,
            invoice_no,
            invoice_date,
            vendor_id,
            vendor_name,
            trans_id,
            item_id,
            item_name,
            qty,
            unit_name,
            price,
            ext_cost,
        ) = row.map_err(|e| e.to_string())?;
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            invoice_id,
            invoice_no,
            invoice_date,
            vendor_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string()),
            vendor_name,
            trans_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string()),
            item_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string()),
            item_name,
            qty.map(|v| v.to_string()).unwrap_or_else(|| "".to_string()),
            unit_name,
            price
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string()),
            ext_cost
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string()),
        )
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    Ok(ExportCsvResponse { path, rows: count })
}

#[tauri::command]
fn patch_convunit(
    app: tauri::AppHandle,
    base_path: String,
    item_id: i64,
    vendor_id: i64,
    unit_id1: i64,
    unit_id2: i64,
    qty1: f64,
    qty2: f64,
) -> Result<PatchResponse, String> {
    let base = resolve_base_path(&base_path)?;
    if !base.exists() {
        return Err(format!("Base path does not exist: {}", base.display()));
    }

    let patch_path = base.join("ConvUnit_patch.csv");
    let new_file = !patch_path.exists();
    if let Some(parent) = patch_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&patch_path)
        .map_err(|e| e.to_string())?;
    if new_file {
        writeln!(
            file,
            "ItemID,VendorID,UnitID1,UnitID2,Quantity1,Quantity2,Status,IsCalculated"
        )
        .map_err(|e| e.to_string())?;
    }
    writeln!(
        file,
        "{},{},{},{},{},{},,1",
        item_id, vendor_id, unit_id1, unit_id2, qty1, qty2
    )
    .map_err(|e| e.to_string())?;

    let conn = open_initialized_db(&app)?;
    conn.execute(
        "INSERT INTO convunit (item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, status, is_calculated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (item_id, vendor_id, unit_id1, unit_id2, qty1, qty2, None::<i64>, Some(1i64)),
    )
    .map_err(|e| e.to_string())?;

    Ok(PatchResponse {
        ok: true,
        message: format!("Patched {}", patch_path.display()),
    })
}

#[derive(Serialize)]
struct BrowseTableResponse {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    total: i64,
}

const ALLOWED_TABLES: &[&str] = &[
    "units",
    "items",
    "vendors",
    "recipes",
    "recp_items",
    "convunit",
    "inv_units",
    "inv_prices",
    "conv_suggestions",
    "conv_suggestions_safe",
    "conv_todo",
    "missing_edges",
    "missing_purch_unit",
    "missing_data_report",
    "invoices",
    "trans",
    "recp_inv",
    "bids",
];

#[tauri::command]
fn browse_table(
    app: tauri::AppHandle,
    table_name: String,
    limit: u32,
    offset: u32,
) -> Result<BrowseTableResponse, String> {
    if !ALLOWED_TABLES.contains(&table_name.as_str()) {
        return Err(format!("Table '{}' is not browsable", table_name));
    }

    let conn = open_initialized_db(&app)?;

    let total: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?;

    let query = format!(
        "SELECT * FROM {} LIMIT {} OFFSET {}",
        table_name, limit, offset
    );
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let columns: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect();

    let col_count = columns.len();
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            let mut vals = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let val: String = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => String::new(),
                    Ok(rusqlite::types::ValueRef::Integer(n)) => n.to_string(),
                    Ok(rusqlite::types::ValueRef::Real(f)) => format!("{:.4}", f),
                    Ok(rusqlite::types::ValueRef::Text(t)) => {
                        String::from_utf8_lossy(t).to_string()
                    }
                    Ok(rusqlite::types::ValueRef::Blob(b)) => format!("[blob {} bytes]", b.len()),
                    Err(_) => String::new(),
                };
                vals.push(val);
            }
            Ok(vals)
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(BrowseTableResponse {
        columns,
        rows,
        total,
    })
}

// ──────────────────────────────────────────
// Export helpers: PDF + DOCX for inventory & recipes
// ──────────────────────────────────────────

use printpdf::*;
use std::io::BufWriter;

/// Wraps long text into lines of at most `max_chars` characters, breaking on
/// whitespace boundaries.
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if raw_line.len() <= max_chars {
            lines.push(raw_line.to_string());
            continue;
        }
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            if current.is_empty() {
                current = word.to_string();
            } else if current.len() + 1 + word.len() <= max_chars {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn fmt_money(v: f64) -> String {
    format!("${:.2}", v)
}

#[tauri::command]
fn export_inventory_pdf(
    app: tauri::AppHandle,
    output_path: String,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;

    let mut stmt = conn
        .prepare("SELECT item_id, name, COALESCE(status, 0) FROM items ORDER BY name COLLATE NOCASE")
        .map_err(|e| e.to_string())?;
    let items: Vec<(i64, String, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let (doc, page1, layer1) = PdfDocument::new("4chef Inventory", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 280.0f32;
    let line_height = 5.0f32;
    let margin_left = 15.0f32;

    // Title
    current_layer.use_text("4chef — Inventory Report", 16.0, Mm(margin_left), Mm(y), &font_bold);
    y -= 10.0;

    // Column headers
    current_layer.use_text("ID", 9.0, Mm(margin_left), Mm(y), &font_bold);
    current_layer.use_text("Name", 9.0, Mm(margin_left + 20.0), Mm(y), &font_bold);
    current_layer.use_text("Status", 9.0, Mm(margin_left + 150.0), Mm(y), &font_bold);
    y -= 3.0;

    // Draw a line
    let line_pts = vec![
        (printpdf::Point::new(Mm(margin_left), Mm(y)), false),
        (printpdf::Point::new(Mm(195.0), Mm(y)), false),
    ];
    current_layer.add_line(printpdf::Line {
        points: line_pts,
        is_closed: false,
    });
    y -= line_height;

    for (item_id, name, status) in &items {
        if y < 15.0 {
            let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            current_layer = doc.get_page(new_page).get_layer(new_layer);
            y = 285.0;
        }
        current_layer.use_text(&item_id.to_string(), 8.0, Mm(margin_left), Mm(y), &font);
        let display_name = if name.len() > 60 { &name[..60] } else { name.as_str() };
        current_layer.use_text(display_name, 8.0, Mm(margin_left + 20.0), Mm(y), &font);
        current_layer.use_text(&status.to_string(), 8.0, Mm(margin_left + 150.0), Mm(y), &font);
        y -= line_height;
    }

    let file = std::fs::File::create(&output_path).map_err(|e| e.to_string())?;
    doc.save(&mut BufWriter::new(file)).map_err(|e| e.to_string())?;

    Ok(PatchResponse {
        ok: true,
        message: format!("Exported {} items to {}", items.len(), output_path),
    })
}

#[tauri::command]
fn export_recipe_pdf(
    app: tauri::AppHandle,
    recipe_id: i64,
    output_path: String,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;

    let (name, instructions): (String, String) = conn
        .query_row(
            "SELECT name, COALESCE(instructions, '') FROM recipes WHERE recipe_id = ?1",
            [recipe_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT ri.item_id, COALESCE(i.name, '?'), ri.qty, COALESCE(u.sing, '?')
             FROM recp_items ri
             LEFT JOIN items i ON i.item_id = ri.item_id
             LEFT JOIN units u ON u.unit_id = ri.unit_id
             WHERE ri.recipe_id = ?1
             ORDER BY ri.recp_item_id",
        )
        .map_err(|e| e.to_string())?;
    let ingredients: Vec<(i64, String, Option<f64>, String)> = stmt
        .query_map([recipe_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let (doc, page1, layer1) =
        PdfDocument::new(&format!("Recipe — {}", name), Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 280.0f32;
    let margin_left = 15.0f32;

    // Title
    current_layer.use_text(&name, 18.0, Mm(margin_left), Mm(y), &font_bold);
    y -= 10.0;

    // Instructions
    if !instructions.trim().is_empty() {
        current_layer.use_text("Instructions", 11.0, Mm(margin_left), Mm(y), &font_bold);
        y -= 6.0;
        for line in wrap_text(&instructions, 90) {
            if y < 15.0 {
                let (np, nl) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                current_layer = doc.get_page(np).get_layer(nl);
                y = 285.0;
            }
            current_layer.use_text(&line, 9.0, Mm(margin_left), Mm(y), &font);
            y -= 5.0;
        }
        y -= 4.0;
    }

    // Ingredients header
    current_layer.use_text("Ingredients", 11.0, Mm(margin_left), Mm(y), &font_bold);
    y -= 7.0;
    current_layer.use_text("Item", 9.0, Mm(margin_left), Mm(y), &font_bold);
    current_layer.use_text("Qty", 9.0, Mm(margin_left + 100.0), Mm(y), &font_bold);
    current_layer.use_text("Unit", 9.0, Mm(margin_left + 130.0), Mm(y), &font_bold);
    y -= 3.0;
    let line_pts = vec![
        (printpdf::Point::new(Mm(margin_left), Mm(y)), false),
        (printpdf::Point::new(Mm(195.0), Mm(y)), false),
    ];
    current_layer.add_line(printpdf::Line {
        points: line_pts,
        is_closed: false,
    });
    y -= 5.0;

    for (_, item_name, qty, unit) in &ingredients {
        if y < 15.0 {
            let (np, nl) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            current_layer = doc.get_page(np).get_layer(nl);
            y = 285.0;
        }
        let display_name = if item_name.len() > 50 { &item_name[..50] } else { item_name.as_str() };
        current_layer.use_text(display_name, 8.0, Mm(margin_left), Mm(y), &font);
        let qty_str = qty.map(|q| format!("{:.2}", q)).unwrap_or_else(|| "-".to_string());
        current_layer.use_text(&qty_str, 8.0, Mm(margin_left + 100.0), Mm(y), &font);
        current_layer.use_text(unit, 8.0, Mm(margin_left + 130.0), Mm(y), &font);
        y -= 5.0;
    }

    let file = std::fs::File::create(&output_path).map_err(|e| e.to_string())?;
    doc.save(&mut BufWriter::new(file)).map_err(|e| e.to_string())?;

    Ok(PatchResponse {
        ok: true,
        message: format!("Exported recipe '{}' to {}", name, output_path),
    })
}

#[tauri::command]
fn export_inventory_docx(
    app: tauri::AppHandle,
    output_path: String,
) -> Result<PatchResponse, String> {
    use docx_rs::*;

    let conn = open_initialized_db(&app)?;
    let mut stmt = conn
        .prepare("SELECT item_id, name, COALESCE(status, 0) FROM items ORDER BY name COLLATE NOCASE")
        .map_err(|e| e.to_string())?;
    let items: Vec<(i64, String, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut docx = Docx::new();

    // Title
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("4chef — Inventory Report").bold().size(32)),
    );
    docx = docx.add_paragraph(Paragraph::new()); // spacer

    // Build table
    let header_row = TableRow::new(vec![
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("ID").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("Name").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("Status").bold())),
    ]);
    let mut rows = vec![header_row];
    for (item_id, name, status) in &items {
        rows.push(TableRow::new(vec![
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(item_id.to_string()))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(name))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(status.to_string()))),
        ]));
    }
    docx = docx.add_table(Table::new(rows));

    let file = std::fs::File::create(&output_path).map_err(|e| e.to_string())?;
    docx.build().pack(file).map_err(|e| e.to_string())?;

    Ok(PatchResponse {
        ok: true,
        message: format!("Exported {} items to {}", items.len(), output_path),
    })
}

#[tauri::command]
fn export_recipe_docx(
    app: tauri::AppHandle,
    recipe_id: i64,
    output_path: String,
) -> Result<PatchResponse, String> {
    use docx_rs::*;

    let conn = open_initialized_db(&app)?;

    let (name, instructions): (String, String) = conn
        .query_row(
            "SELECT name, COALESCE(instructions, '') FROM recipes WHERE recipe_id = ?1",
            [recipe_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT ri.item_id, COALESCE(i.name, '?'), ri.qty, COALESCE(u.sing, '?')
             FROM recp_items ri
             LEFT JOIN items i ON i.item_id = ri.item_id
             LEFT JOIN units u ON u.unit_id = ri.unit_id
             WHERE ri.recipe_id = ?1
             ORDER BY ri.recp_item_id",
        )
        .map_err(|e| e.to_string())?;
    let ingredients: Vec<(i64, String, Option<f64>, String)> = stmt
        .query_map([recipe_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut docx = Docx::new();

    // Title
    docx = docx.add_paragraph(
        Paragraph::new().add_run(Run::new().add_text(&name).bold().size(36)),
    );

    // Instructions
    if !instructions.trim().is_empty() {
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Instructions").bold().size(24)),
        );
        for line in instructions.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(trimmed)),
                );
            }
        }
        docx = docx.add_paragraph(Paragraph::new()); // spacer
    }

    // Ingredients table
    docx = docx.add_paragraph(
        Paragraph::new().add_run(Run::new().add_text("Ingredients").bold().size(24)),
    );
    let header_row = TableRow::new(vec![
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("Item").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("Qty").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("Unit").bold())),
    ]);
    let mut rows = vec![header_row];
    for (_, item_name, qty, unit) in &ingredients {
        let qty_str = qty.map(|q| format!("{:.2}", q)).unwrap_or_else(|| "-".to_string());
        rows.push(TableRow::new(vec![
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(item_name))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(qty_str))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(unit))),
        ]));
    }
    docx = docx.add_table(Table::new(rows));

    let file = std::fs::File::create(&output_path).map_err(|e| e.to_string())?;
    docx.build().pack(file).map_err(|e| e.to_string())?;

    Ok(PatchResponse {
        ok: true,
        message: format!("Exported recipe '{}' to {}", name, output_path),
    })
}

// ──────────────────────────────────────────
// PDF Invoice Import — extract text-based PDF invoices
// ──────────────────────────────────────────

#[derive(Serialize)]
struct PdfInvoiceLine {
    item_name: String,
    qty: Option<f64>,
    unit: String,
    unit_price: Option<f64>,
    ext_cost: Option<f64>,
}

#[derive(Serialize)]
struct PdfInvoicePreview {
    vendor_guess: String,
    invoice_no: String,
    invoice_date: String,
    lines: Vec<PdfInvoiceLine>,
    raw_text: String,
}

/// Try to parse a dollar value from a string like "$12.50" or "12.50".
fn parse_dollar(s: &str) -> Option<f64> {
    let cleaned = s.trim().trim_start_matches('$').replace(',', "");
    cleaned.parse::<f64>().ok()
}

#[tauri::command]
fn preview_pdf_invoice(pdf_path: String) -> Result<PdfInvoicePreview, String> {
    let bytes = std::fs::read(&pdf_path).map_err(|e| format!("Cannot read file: {e}"))?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| format!("PDF extraction failed: {e}"))?;

    // Heuristic parsing for common invoice formats
    let mut vendor_guess = String::new();
    let mut invoice_no = String::new();
    let mut invoice_date = String::new();
    let mut lines = Vec::new();

    let text_lines: Vec<&str> = text.lines().collect();

    // First non-empty line is often the vendor name
    for line in &text_lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && vendor_guess.is_empty() {
            vendor_guess = trimmed.to_string();
            break;
        }
    }

    for line in &text_lines {
        let lower = line.to_lowercase();
        // Invoice number
        if invoice_no.is_empty() {
            if let Some(pos) = lower.find("invoice") {
                let after = &line[pos..];
                let parts: Vec<&str> = after.split_whitespace().collect();
                // Look for a number-like token after "invoice"
                for part in parts.iter().skip(1) {
                    let cleaned = part.trim_matches(|c: char| !c.is_alphanumeric());
                    if !cleaned.is_empty() {
                        invoice_no = cleaned.to_string();
                        break;
                    }
                }
            }
        }
        // Date (look for patterns like MM/DD/YYYY or YYYY-MM-DD)
        if invoice_date.is_empty() {
            if let Some(pos) = lower.find("date") {
                let after = &line[pos..];
                let parts: Vec<&str> = after.split_whitespace().collect();
                for part in parts.iter().skip(1) {
                    let cleaned = part.trim_matches(|c: char| c == ':' || c == ' ');
                    if cleaned.len() >= 8 && cleaned.contains('/') || cleaned.contains('-') {
                        invoice_date = cleaned.to_string();
                        break;
                    }
                }
            }
        }

        // Try to extract line items — look for lines with dollar amounts
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() >= 3 {
            // Find dollar-like values from the right
            let mut dollar_vals: Vec<f64> = Vec::new();
            let mut name_end = tokens.len();
            for (i, token) in tokens.iter().enumerate().rev() {
                if let Some(val) = parse_dollar(token) {
                    dollar_vals.push(val);
                    name_end = i;
                } else if !dollar_vals.is_empty() {
                    break;
                }
            }
            // Also look for a qty at the beginning or near the dollar values
            if !dollar_vals.is_empty() && name_end > 0 {
                dollar_vals.reverse();
                let name_parts: Vec<&str> = tokens[..name_end].to_vec();
                // Try to find qty — first numeric token in name_parts from the left or right
                let mut qty: Option<f64> = None;
                let mut unit = String::new();
                let mut actual_name_parts = name_parts.clone();

                // Check if first token is a number (qty)
                if let Ok(q) = name_parts[0].parse::<f64>() {
                    qty = Some(q);
                    actual_name_parts = name_parts[1..].to_vec();
                    // Check if second token looks like a unit (short word)
                    if actual_name_parts.len() > 1 && actual_name_parts[0].len() <= 5 {
                        unit = actual_name_parts[0].to_string();
                        actual_name_parts = actual_name_parts[1..].to_vec();
                    }
                }

                let item_name = actual_name_parts.join(" ");
                if !item_name.is_empty() && item_name.len() > 2 {
                    let unit_price = if dollar_vals.len() >= 2 {
                        Some(dollar_vals[0])
                    } else {
                        None
                    };
                    let ext_cost = dollar_vals.last().copied();
                    lines.push(PdfInvoiceLine {
                        item_name,
                        qty,
                        unit,
                        unit_price,
                        ext_cost,
                    });
                }
            }
        }
    }

    Ok(PdfInvoicePreview {
        vendor_guess,
        invoice_no,
        invoice_date,
        lines,
        raw_text: text,
    })
}

#[tauri::command]
fn import_pdf_invoice(
    app: tauri::AppHandle,
    vendor_id: i64,
    invoice_no: String,
    invoice_date: String,
    lines: Vec<serde_json::Value>,
) -> Result<PatchResponse, String> {
    let conn = open_initialized_db(&app)?;

    // Find next invoice_id
    let next_id: i64 = conn
        .query_row("SELECT COALESCE(MAX(invoice_id), 0) + 1 FROM invoices", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    // Insert invoice header
    conn.execute(
        "INSERT INTO invoices (status, invoice_id, invoice_date, vendor_id, invoice_no, freight, total)
         VALUES (1, ?1, ?2, ?3, ?4, 0, 0)",
        (next_id, &invoice_date, vendor_id, &invoice_no),
    )
    .map_err(|e| e.to_string())?;

    let mut trans_count = 0i64;
    let next_trans_base: i64 = conn
        .query_row("SELECT COALESCE(MAX(trans_id), 0) + 1 FROM trans", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let mut total = 0.0f64;
    for (i, line) in lines.iter().enumerate() {
        let item_name = line.get("item_name").and_then(|v| v.as_str()).unwrap_or("");
        let qty = line.get("qty").and_then(|v| v.as_f64());
        let unit_price = line.get("unit_price").and_then(|v| v.as_f64());
        let ext_cost = line.get("ext_cost").and_then(|v| v.as_f64());

        if item_name.is_empty() {
            continue;
        }

        // Try to match item by name
        let item_id: Option<i64> = conn
            .query_row(
                "SELECT item_id FROM items WHERE LOWER(name) = LOWER(?1) LIMIT 1",
                [item_name],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        // If no match, create a new item
        let final_item_id = match item_id {
            Some(id) => id,
            None => {
                let new_id: i64 = conn
                    .query_row("SELECT COALESCE(MAX(item_id), 0) + 1 FROM items", [], |r| r.get(0))
                    .map_err(|e| e.to_string())?;
                conn.execute(
                    "INSERT INTO items (item_id, name, status) VALUES (?1, ?2, 1)",
                    (new_id, item_name),
                )
                .map_err(|e| e.to_string())?;
                new_id
            }
        };

        let line_cost = ext_cost.unwrap_or(0.0);
        total += line_cost;

        let trans_id = next_trans_base + i as i64;
        conn.execute(
            "INSERT INTO trans (status, invoice_id, trans_id, item_id, trans_date, vendor_id, price, qty, unit_id, ext_cost)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8)",
            (
                next_id,
                trans_id,
                final_item_id,
                &invoice_date,
                vendor_id,
                unit_price.unwrap_or(0.0),
                qty.unwrap_or(0.0),
                line_cost,
            ),
        )
        .map_err(|e| e.to_string())?;
        trans_count += 1;
    }

    // Update invoice total
    conn.execute(
        "UPDATE invoices SET total = ?2 WHERE invoice_id = ?1",
        (next_id, total),
    )
    .map_err(|e| e.to_string())?;

    Ok(PatchResponse {
        ok: true,
        message: format!(
            "Imported invoice #{} with {} lines (total {})",
            invoice_no,
            trans_count,
            fmt_money(total)
        ),
    })
}

// ── Food cost calculator ──

#[derive(serde::Deserialize)]
struct FoodCostLineInput {
    item_id: i64,
    unit_id: Option<i64>,
    qty: f64,
}

#[derive(Serialize)]
struct FoodCostLineOutput {
    item_id: i64,
    item_name: String,
    unit_id: Option<i64>,
    unit_name: String,
    qty: f64,
    price: Option<f64>,
    extended_cost: Option<f64>,
    cost_status: String,
}

#[derive(Serialize)]
struct FoodCostResponse {
    lines: Vec<FoodCostLineOutput>,
    total_cost: f64,
    missing_costs: i64,
}

#[tauri::command]
fn calculate_food_cost(
    app: tauri::AppHandle,
    lines: Vec<FoodCostLineInput>,
) -> Result<FoodCostResponse, String> {
    let conn = open_initialized_db(&app)?;

    let mut out_lines = Vec::new();
    let mut total_cost = 0.0f64;
    let mut missing_costs = 0i64;

    for line in &lines {
        let item_name: String = conn
            .query_row(
                "SELECT name FROM items WHERE item_id = ?1",
                [line.item_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "(unknown)".to_string());

        let unit_name: String = if let Some(uid) = line.unit_id {
            conn.query_row("SELECT sing FROM units WHERE unit_id = ?1", [uid], |row| {
                row.get(0)
            })
            .unwrap_or_else(|_| "-".to_string())
        } else {
            "-".to_string()
        };

        let purch_unit_id: Option<i64> = conn
            .query_row(
                "SELECT purch_unit_id FROM inv_units WHERE item_id = ?1 AND is_default = 1 LIMIT 1",
                [line.item_id],
                |row| row.get(0),
            )
            .ok()
            .or_else(|| {
                conn.query_row(
                    "SELECT purch_unit_id FROM inv_units WHERE item_id = ?1 LIMIT 1",
                    [line.item_id],
                    |row| row.get(0),
                )
                .ok()
            });

        let price_vendor: Option<(f64, i64)> = conn
            .query_row(
                "SELECT price, vendor_id FROM inv_prices WHERE item_id = ?1 ORDER BY vendor_id LIMIT 1",
                [line.item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok()
            .or_else(|| {
                conn.query_row(
                    "SELECT price, vendor_id FROM trans WHERE item_id = ?1 AND price IS NOT NULL AND price > 0 ORDER BY trans_date DESC, trans_id DESC LIMIT 1",
                    [line.item_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok()
            });
        let price = price_vendor.map(|v| v.0);
        let price_vendor_id = price_vendor.map(|v| v.1);

        let mut factor: Option<f64> = None;
        if let (Some(recipe_unit_id), Some(pu_id)) = (line.unit_id, purch_unit_id) {
            if recipe_unit_id == pu_id {
                factor = Some(1.0);
            } else {
                let edges = build_conversion_edges(&conn, line.item_id, price_vendor_id)?;
                if let Some((f, _)) = bfs_conversion_factor(&edges, recipe_unit_id, pu_id, 6) {
                    factor = Some(f);
                }
            }
        }

        let (cost_status, extended_cost) = if line.qty <= 0.0 {
            ("Missing qty".to_string(), None)
        } else if purch_unit_id.is_none() {
            ("Missing purch unit".to_string(), None)
        } else if price.is_none() {
            ("Missing price".to_string(), None)
        } else if factor.is_none() {
            ("Needs conversion".to_string(), None)
        } else {
            let cost = line.qty * factor.unwrap_or(1.0) * price.unwrap_or(0.0);
            ("OK".to_string(), Some(cost))
        };

        if let Some(c) = extended_cost {
            total_cost += c;
        } else {
            missing_costs += 1;
        }

        out_lines.push(FoodCostLineOutput {
            item_id: line.item_id,
            item_name,
            unit_id: line.unit_id,
            unit_name,
            qty: line.qty,
            price,
            extended_cost,
            cost_status,
        });
    }

    Ok(FoodCostResponse {
        lines: out_lines,
        total_cost,
        missing_costs,
    })
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_db_path,
            get_import_summary,
            import_exports,
            import_from_mdf,
            auto_ingest_invoices,
            auto_ingest_sysco_invoices,
            revert_db,
            recalculate_reports,
            open_path,
            get_inventory_detail,
            search_vendors,
            get_vendor_detail,
            list_vendors_simple,
            list_vendors_all,
            list_units_simple,
            list_items_simple,
            search_inventory,
            search_recipes,
            get_recipe_detail,
            get_conversion_overview,
            list_conv_suggestions,
            list_conv_todo,
            list_missing_edges,
            list_missing_purch_unit,
            list_missing_data_report,
            list_invoices,
            get_invoice_detail,
            export_invoice_lines_csv,
            patch_convunit,
            set_item_purch_unit,
            upsert_manual_price,
            update_item,
            update_vendor,
            update_recipe,
            add_recp_item,
            upsert_convunit,
            update_invoice,
            update_trans_line,
            merge_vendor,
            ping_backend,
            browse_table,
            export_inventory_pdf,
            export_inventory_docx,
            export_recipe_pdf,
            export_recipe_docx,
            preview_pdf_invoice,
            import_pdf_invoice,
            calculate_food_cost
        ])
        .setup(|app| {
            let path = db_path(&app.handle())?;
            if let Ok(conn) = open_db(&path) {
                let _ = init_db(&conn);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
