use js_sys::{Promise, Reflect};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> Promise;
}

#[derive(Serialize)]
struct ImportArgs {
    #[serde(rename = "basePath")]
    base_path: String,
}

#[derive(Serialize)]
struct ImportMdfArgs {
    #[serde(rename = "mdfPath")]
    mdf_path: String,
}

#[derive(Serialize)]
struct InventoryQueryArgs {
    query: String,
    limit: u32,
    offset: u32,
}

#[derive(Serialize)]
struct RecipeQueryArgs {
    query: String,
    limit: u32,
    offset: u32,
}

#[derive(Serialize)]
struct RecipeDetailArgs {
    #[serde(rename = "recipeId")]
    recipe_id: i64,
}

#[derive(Serialize)]
struct PingArgs {}

#[derive(Serialize)]
struct OpenPathArgs {
    path: String,
}

#[derive(Serialize)]
struct ExportInvoiceArgs {
    query: String,
    #[serde(rename = "vendorId")]
    vendor_id: Option<i64>,
    #[serde(rename = "dateFrom")]
    date_from: String,
    #[serde(rename = "dateTo")]
    date_to: String,
    #[serde(rename = "outputPath")]
    output_path: Option<String>,
}

#[derive(Serialize)]
struct PatchConvunitArgs {
    #[serde(rename = "basePath")]
    base_path: String,
    #[serde(rename = "itemId")]
    item_id: i64,
    #[serde(rename = "vendorId")]
    vendor_id: i64,
    #[serde(rename = "unitId1")]
    unit_id1: i64,
    #[serde(rename = "unitId2")]
    unit_id2: i64,
    #[serde(rename = "qty1")]
    qty1: f64,
    #[serde(rename = "qty2")]
    qty2: f64,
}

#[derive(Serialize)]
struct UpsertConvunitArgs {
    #[serde(rename = "itemId")]
    item_id: i64,
    #[serde(rename = "vendorId")]
    vendor_id: i64,
    #[serde(rename = "unitId1")]
    unit_id1: i64,
    #[serde(rename = "unitId2")]
    unit_id2: i64,
    #[serde(rename = "qty1")]
    qty1: f64,
    #[serde(rename = "qty2")]
    qty2: f64,
    status: Option<i64>,
}

#[derive(Serialize)]
struct SetPurchUnitArgs {
    #[serde(rename = "itemId")]
    item_id: i64,
    #[serde(rename = "purchUnitId")]
    purch_unit_id: i64,
    #[serde(rename = "isDefault")]
    is_default: bool,
}

#[derive(Serialize)]
struct UpsertManualPriceArgs {
    #[serde(rename = "itemId")]
    item_id: i64,
    #[serde(rename = "vendorId")]
    vendor_id: i64,
    price: f64,
    pack: String,
}

#[derive(Serialize)]
struct MergeVendorArgs {
    #[serde(rename = "sourceVendorId")]
    source_vendor_id: i64,
    #[serde(rename = "targetVendorId")]
    target_vendor_id: i64,
}

#[derive(Serialize)]
struct BrowseTableArgs {
    #[serde(rename = "tableName")]
    table_name: String,
    limit: u32,
    offset: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct BrowseTableResponse {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    total: i64,
}

// ── Update / export / PDF-import args ──

#[derive(Serialize)]
struct UpdateItemArgs {
    #[serde(rename = "itemId")]
    item_id: i64,
    name: String,
    status: Option<i64>,
}

#[derive(Serialize)]
struct UpdateRecipeArgs {
    #[serde(rename = "recipeId")]
    recipe_id: i64,
    name: String,
    instructions: String,
}

#[derive(Serialize)]
struct AddRecpItemArgs {
    #[serde(rename = "recipeId")]
    recipe_id: i64,
    #[serde(rename = "recpItemId")]
    recp_item_id: Option<i64>,
    #[serde(rename = "itemId")]
    item_id: i64,
    #[serde(rename = "unitId")]
    unit_id: Option<i64>,
    qty: Option<f64>,
}

#[derive(Serialize)]
struct ExportPathArgs {
    #[serde(rename = "outputPath")]
    output_path: String,
}

#[derive(Serialize)]
struct GlobalSearchArgs {
    query: String,
}

#[derive(Deserialize, Clone, Debug)]
struct GlobalSearchHit {
    category: String,
    id: i64,
    label: String,
    detail: String,
}

#[derive(Serialize)]
struct ExportRecipeArgs {
    #[serde(rename = "recipeId")]
    recipe_id: i64,
    #[serde(rename = "outputPath")]
    output_path: String,
}

#[derive(Serialize)]
struct PdfPathArgs {
    #[serde(rename = "pdfPath")]
    pdf_path: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct PdfInvoiceLine {
    item_name: String,
    qty: Option<f64>,
    unit: String,
    unit_price: Option<f64>,
    ext_cost: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct PdfInvoicePreview {
    vendor_guess: String,
    invoice_no: String,
    invoice_date: String,
    lines: Vec<PdfInvoiceLine>,
    raw_text: String,
}

#[derive(Serialize)]
struct ImportPdfInvoiceArgs {
    #[serde(rename = "vendorId")]
    vendor_id: i64,
    #[serde(rename = "invoiceNo")]
    invoice_no: String,
    #[serde(rename = "invoiceDate")]
    invoice_date: String,
    lines: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct ImportSummary {
    db_path: String,
    units: usize,
    items: usize,
    convunit: usize,
    recp_items: usize,
    inv_units: usize,
    inv_prices: usize,
    vendors: usize,
    recipes: usize,
    conv_suggestions: usize,
    conv_suggestions_safe: usize,
    conv_todo: usize,
    missing_edges: usize,
    missing_purch_unit: usize,
    missing_data_report: usize,
    invoices: usize,
    trans: usize,
    recp_inv: usize,
    bids: usize,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InventoryItem {
    item_id: i64,
    name: String,
    status: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InventoryQueryResponse {
    items: Vec<InventoryItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize)]
struct InventoryDetailArgs {
    #[serde(rename = "itemId")]
    item_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InventoryDetailUnit {
    unit_id: Option<i64>,
    unit_name: String,
    is_default: Option<i64>,
    status: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InventoryDetailPrice {
    vendor_id: Option<i64>,
    vendor_name: String,
    price: Option<f64>,
    prev_price: Option<f64>,
    diff_pct: Option<f64>,
    pack: String,
    status: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InventoryDetailConversion {
    vendor_id: i64,
    unit_id1: i64,
    unit_id2: i64,
    qty1: f64,
    qty2: f64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InventoryDetailUsage {
    recipe_id: i64,
    recipe_name: String,
    qty: Option<f64>,
    unit_name: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InventoryDetailMissingEdge {
    vendor_id: i64,
    recipe_unit: String,
    purch_unit: String,
    hits: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize)]
struct VendorQueryArgs {
    query: String,
    limit: u32,
    offset: u32,
}

#[derive(Serialize)]
struct VendorDetailArgs {
    #[serde(rename = "vendorId")]
    vendor_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct VendorListItem {
    vendor_id: i64,
    name: String,
    price_items: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct VendorListResponse {
    vendors: Vec<VendorListItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct VendorPriceItem {
    item_id: i64,
    item_name: String,
    price: Option<f64>,
    pack: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct VendorDetailResponse {
    vendor_id: i64,
    name: String,
    price_items: Vec<VendorPriceItem>,
    invoice_count: i64,
    trans_count: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct VendorSimple {
    vendor_id: i64,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct VendorSimpleResponse {
    vendors: Vec<VendorSimple>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct UnitSimple {
    unit_id: i64,
    sing: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct UnitSimpleResponse {
    units: Vec<UnitSimple>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct RecipeListItem {
    recipe_id: i64,
    name: String,
    item_count: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct RecipeListResponse {
    recipes: Vec<RecipeListItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
struct RecipeDetailResponse {
    recipe_id: i64,
    name: String,
    item_count: i64,
    total_cost: f64,
    missing_costs: i64,
    instructions: String,
    ingredients: Vec<RecipeIngredient>,
    #[serde(default)]
    allergens: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct ConversionOverview {
    suggestions: i64,
    suggestions_safe: i64,
    todo: i64,
    missing_edges: i64,
    missing_purch: i64,
    missing_data: i64,
}

#[derive(Serialize)]
struct ConversionListArgs {
    table: String,
    limit: u32,
    offset: u32,
}

#[derive(Serialize)]
struct ConversionPageArgs {
    limit: u32,
    offset: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
struct ConversionSuggestionResponse {
    rows: Vec<ConversionSuggestionRow>,
    total: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
struct ConversionTodoResponse {
    rows: Vec<ConversionTodoRow>,
    total: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
struct MissingEdgeResponse {
    rows: Vec<MissingEdgeRow>,
    total: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct MissingPurchRow {
    item_id: i64,
    item_name: String,
    usage_count: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct MissingPurchResponse {
    rows: Vec<MissingPurchRow>,
    total: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct MissingDataRow {
    recipe_id: i64,
    recipe_name: String,
    missing_a: Option<i64>,
    missing_b: Option<i64>,
    missing_c: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct MissingDataResponse {
    rows: Vec<MissingDataRow>,
    total: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InvoiceListItem {
    invoice_id: i64,
    invoice_no: String,
    vendor_id: Option<i64>,
    vendor_name: String,
    invoice_date: String,
    total: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InvoiceListResponse {
    invoices: Vec<InvoiceListItem>,
    total: i64,
    filtered: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InvoiceLineItem {
    trans_id: Option<i64>,
    item_id: Option<i64>,
    item_name: String,
    qty: Option<f64>,
    unit_name: String,
    price: Option<f64>,
    ext_cost: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InvoiceDetailResponse {
    invoice: InvoiceListItem,
    freight: Option<f64>,
    lines: Vec<InvoiceLineItem>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExportCsvResponse {
    path: String,
    rows: usize,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct PatchResponse {
    ok: bool,
    message: String,
}

#[derive(Serialize)]
struct InvoiceQueryArgs {
    query: String,
    #[serde(rename = "vendorId")]
    vendor_id: Option<i64>,
    #[serde(rename = "dateFrom")]
    date_from: String,
    #[serde(rename = "dateTo")]
    date_to: String,
    limit: u32,
    offset: u32,
}

#[derive(Serialize)]
struct InvoiceDetailArgs {
    #[serde(rename = "invoiceId")]
    invoice_id: i64,
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct PingResponse {
    ok: bool,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct ItemSimple {
    item_id: i64,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct ItemSimpleResponse {
    items: Vec<ItemSimple>,
}

#[derive(Serialize, Clone)]
struct FoodCostLineInput {
    #[serde(rename = "itemId")]
    item_id: i64,
    #[serde(rename = "unitId")]
    unit_id: Option<i64>,
    qty: f64,
}

#[derive(Serialize)]
struct SaveSettingsArgs {
    #[serde(rename = "companyName")]
    company_name: String,
    #[serde(rename = "serviceCategory")]
    service_category: String,
    #[serde(rename = "operationSize")]
    operation_size: String,
}

#[derive(Serialize)]
struct UploadLogoArgs {
    #[serde(rename = "sourcePath")]
    source_path: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct SettingsResponse {
    company_name: String,
    logo_path: String,
    service_category: String,
    operation_size: String,
}

#[derive(Serialize)]
struct CalculateFoodCostArgs {
    lines: Vec<FoodCostLineInput>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
struct FoodCostResponse {
    lines: Vec<FoodCostLineOutput>,
    total_cost: f64,
    missing_costs: i64,
}

async fn invoke_cmd<T: for<'de> Deserialize<'de>>(cmd: &str, args: JsValue) -> Result<T, String> {
    let promise = invoke(cmd, args);
    match JsFuture::from(promise).await {
        Ok(value) => serde_wasm_bindgen::from_value(value).map_err(|e| e.to_string()),
        Err(err) => {
            if let Some(s) = err.as_string() {
                return Err(s);
            }

            if let Some(js_err) = err.dyn_ref::<js_sys::Error>() {
                return Err(js_err.message().into());
            }

            let msg = Reflect::get(&err, &JsValue::from_str("message"))
                .ok()
                .and_then(|v| v.as_string());
            if let Some(msg) = msg {
                return Err(msg);
            }

            let msg = Reflect::get(&err, &JsValue::from_str("error"))
                .ok()
                .and_then(|v| v.as_string());
            if let Some(msg) = msg {
                return Err(msg);
            }

            let msg = Reflect::get(&err, &JsValue::from_str("cause"))
                .ok()
                .and_then(|v| v.as_string());
            if let Some(msg) = msg {
                return Err(msg);
            }

            if err.is_object() {
                let obj = js_sys::Object::from(err.clone());
                let keys = js_sys::Object::keys(&obj);
                let mut parts = Vec::new();
                for idx in 0..keys.length() {
                    if let Some(key) = keys.get(idx).as_string() {
                        if let Ok(value) = Reflect::get(&obj, &JsValue::from_str(&key)) {
                            if let Some(v) = value.as_string() {
                                parts.push(format!("{key}: {v}"));
                            }
                        }
                    }
                }
                if !parts.is_empty() {
                    return Err(parts.join(" | "));
                }
            }

            let msg = js_sys::JSON::stringify(&err)
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Command failed".to_string());
            Err(msg)
        }
    }
}

fn format_money(value: f64) -> String {
    format!("${:.2}", value)
}

fn trigger_inventory_fetch(
    query: String,
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_items: WriteSignal<Vec<InventoryItem>>,
    set_total: WriteSignal<i64>,
    set_filtered: WriteSignal<i64>,
    set_loaded: WriteSignal<bool>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading inventory...".to_string());

    spawn_local(async move {
        let args = to_value(&InventoryQueryArgs {
            query,
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();

        match invoke_cmd::<InventoryQueryResponse>("search_inventory", args).await {
            Ok(result) => {
                set_items.set(result.items);
                set_total.set(result.total);
                set_filtered.set(result.filtered);
                set_status.set("Inventory loaded".to_string());
                set_loaded.set(true);
            }
            Err(err) => {
                set_status.set(format!("Inventory load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_recipe_fetch(
    query: String,
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_list: WriteSignal<Vec<RecipeListItem>>,
    set_total: WriteSignal<i64>,
    set_filtered: WriteSignal<i64>,
    set_loaded: WriteSignal<bool>,
    auto_select: bool,
    set_detail_loading: WriteSignal<bool>,
    set_detail_status: WriteSignal<String>,
    set_selected: WriteSignal<Option<RecipeDetailResponse>>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading recipes...".to_string());

    spawn_local(async move {
        let args = to_value(&RecipeQueryArgs {
            query,
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();

        match invoke_cmd::<RecipeListResponse>("search_recipes", args).await {
            Ok(result) => {
                let first_id = result.recipes.first().map(|r| r.recipe_id);
                set_list.set(result.recipes);
                set_total.set(result.total);
                set_filtered.set(result.filtered);
                set_status.set("Recipes loaded".to_string());
                set_loaded.set(true);
                if auto_select {
                    if let Some(recipe_id) = first_id {
                        trigger_recipe_detail_fetch(
                            recipe_id,
                            set_detail_loading,
                            set_detail_status,
                            set_selected,
                        );
                    }
                }
            }
            Err(err) => {
                set_status.set(format!("Recipe load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_recipe_detail_fetch(
    recipe_id: i64,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_selected: WriteSignal<Option<RecipeDetailResponse>>,
) {
    set_loading.set(true);
    set_status.set("Loading recipe...".to_string());

    spawn_local(async move {
        let args = to_value(&RecipeDetailArgs { recipe_id }).unwrap();
        match invoke_cmd::<RecipeDetailResponse>("get_recipe_detail", args).await {
            Ok(result) => {
                set_selected.set(Some(result));
                set_status.set("Recipe loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Recipe load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_inventory_detail_fetch(
    item_id: i64,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_selected: WriteSignal<Option<InventoryDetailResponse>>,
) {
    set_loading.set(true);
    set_status.set("Loading item detail...".to_string());
    spawn_local(async move {
        let args = to_value(&InventoryDetailArgs { item_id }).unwrap();
        match invoke_cmd::<InventoryDetailResponse>("get_inventory_detail", args).await {
            Ok(result) => {
                set_selected.set(Some(result));
                set_status.set("Item detail loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Item detail failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_vendor_fetch(
    query: String,
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_list: WriteSignal<Vec<VendorListItem>>,
    set_total: WriteSignal<i64>,
    set_filtered: WriteSignal<i64>,
    set_loaded: WriteSignal<bool>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading vendors...".to_string());
    spawn_local(async move {
        let args = to_value(&VendorQueryArgs {
            query,
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();
        match invoke_cmd::<VendorListResponse>("search_vendors", args).await {
            Ok(result) => {
                set_list.set(result.vendors);
                set_total.set(result.total);
                set_filtered.set(result.filtered);
                set_status.set("Vendors loaded".to_string());
                set_loaded.set(true);
            }
            Err(err) => {
                set_status.set(format!("Vendor load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_vendor_detail_fetch(
    vendor_id: i64,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_selected: WriteSignal<Option<VendorDetailResponse>>,
) {
    set_loading.set(true);
    set_status.set("Loading vendor...".to_string());
    spawn_local(async move {
        let args = to_value(&VendorDetailArgs { vendor_id }).unwrap();
        match invoke_cmd::<VendorDetailResponse>("get_vendor_detail", args).await {
            Ok(result) => {
                set_selected.set(Some(result));
                set_status.set("Vendor loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Vendor load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_vendor_options_fetch(set_options: WriteSignal<Vec<VendorSimple>>) {
    spawn_local(async move {
        let args = to_value(&PingArgs {}).unwrap();
        if let Ok(result) = invoke_cmd::<VendorSimpleResponse>("list_vendors_simple", args).await {
            set_options.set(result.vendors);
        }
    });
}

fn trigger_vendor_all_options_fetch(set_options: WriteSignal<Vec<VendorSimple>>) {
    spawn_local(async move {
        let args = to_value(&PingArgs {}).unwrap();
        if let Ok(result) = invoke_cmd::<VendorSimpleResponse>("list_vendors_all", args).await {
            set_options.set(result.vendors);
        }
    });
}

fn trigger_unit_options_fetch(set_options: WriteSignal<Vec<UnitSimple>>) {
    spawn_local(async move {
        let args = to_value(&PingArgs {}).unwrap();
        if let Ok(result) = invoke_cmd::<UnitSimpleResponse>("list_units_simple", args).await {
            set_options.set(result.units);
        }
    });
}

fn trigger_conversion_overview_fetch(set_overview: WriteSignal<ConversionOverview>) {
    spawn_local(async move {
        let args = to_value(&PingArgs {}).unwrap();
        if let Ok(result) = invoke_cmd::<ConversionOverview>("get_conversion_overview", args).await
        {
            set_overview.set(result);
        }
    });
}

fn trigger_conversion_suggestions_fetch(
    table: String,
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_rows: WriteSignal<Vec<ConversionSuggestionRow>>,
    set_total: WriteSignal<i64>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading conversion suggestions...".to_string());
    spawn_local(async move {
        let args = to_value(&ConversionListArgs {
            table,
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();
        match invoke_cmd::<ConversionSuggestionResponse>("list_conv_suggestions", args).await {
            Ok(result) => {
                set_rows.set(result.rows);
                set_total.set(result.total);
                set_status.set("Suggestions loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Suggestions load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_conversion_todo_fetch(
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_rows: WriteSignal<Vec<ConversionTodoRow>>,
    set_total: WriteSignal<i64>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading conversion todo...".to_string());
    spawn_local(async move {
        let args = to_value(&ConversionPageArgs {
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();
        match invoke_cmd::<ConversionTodoResponse>("list_conv_todo", args).await {
            Ok(result) => {
                set_rows.set(result.rows);
                set_total.set(result.total);
                set_status.set("Todo loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Todo load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_missing_edges_fetch(
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_rows: WriteSignal<Vec<MissingEdgeRow>>,
    set_total: WriteSignal<i64>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading missing edges...".to_string());
    spawn_local(async move {
        let args = to_value(&ConversionPageArgs {
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();
        match invoke_cmd::<MissingEdgeResponse>("list_missing_edges", args).await {
            Ok(result) => {
                set_rows.set(result.rows);
                set_total.set(result.total);
                set_status.set("Missing edges loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Missing edges load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_missing_purch_fetch(
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_rows: WriteSignal<Vec<MissingPurchRow>>,
    set_total: WriteSignal<i64>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading missing purchase units...".to_string());
    spawn_local(async move {
        let args = to_value(&ConversionPageArgs {
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();
        match invoke_cmd::<MissingPurchResponse>("list_missing_purch_unit", args).await {
            Ok(result) => {
                set_rows.set(result.rows);
                set_total.set(result.total);
                set_status.set("Missing purchase units loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Missing purchase units load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_missing_data_fetch(
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_rows: WriteSignal<Vec<MissingDataRow>>,
    set_total: WriteSignal<i64>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading missing data report...".to_string());
    spawn_local(async move {
        let args = to_value(&ConversionPageArgs {
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();
        match invoke_cmd::<MissingDataResponse>("list_missing_data_report", args).await {
            Ok(result) => {
                set_rows.set(result.rows);
                set_total.set(result.total);
                set_status.set("Missing data loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Missing data load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_invoice_fetch(
    query: String,
    vendor_id: Option<i64>,
    date_from: String,
    date_to: String,
    page: usize,
    limit: usize,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_list: WriteSignal<Vec<InvoiceListItem>>,
    set_total: WriteSignal<i64>,
    set_filtered: WriteSignal<i64>,
    set_loaded: WriteSignal<bool>,
) {
    let offset = page * limit;
    set_loading.set(true);
    set_status.set("Loading invoices...".to_string());
    spawn_local(async move {
        let args = to_value(&InvoiceQueryArgs {
            query,
            vendor_id,
            date_from,
            date_to,
            limit: limit as u32,
            offset: offset as u32,
        })
        .unwrap();
        match invoke_cmd::<InvoiceListResponse>("list_invoices", args).await {
            Ok(result) => {
                set_list.set(result.invoices);
                set_total.set(result.total);
                set_filtered.set(result.filtered);
                set_status.set("Invoices loaded".to_string());
                set_loaded.set(true);
            }
            Err(err) => {
                set_status.set(format!("Invoice load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}

fn trigger_invoice_detail_fetch(
    invoice_id: i64,
    set_loading: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_selected: WriteSignal<Option<InvoiceDetailResponse>>,
) {
    set_loading.set(true);
    set_status.set("Loading invoice...".to_string());
    spawn_local(async move {
        let args = to_value(&InvoiceDetailArgs { invoice_id }).unwrap();
        match invoke_cmd::<InvoiceDetailResponse>("get_invoice_detail", args).await {
            Ok(result) => {
                set_selected.set(Some(result));
                set_status.set("Invoice loaded".to_string());
            }
            Err(err) => {
                set_status.set(format!("Invoice load failed: {err}"));
            }
        }
        set_loading.set(false);
    });
}
/// Turns raw SQLite column names into human-friendly display names.
fn humanize_column(raw: &str) -> String {
    match raw {
        // ── common ids / names ──
        "item_id" => "Item ID".into(),
        "unit_id" => "Unit ID".into(),
        "vendor_id" => "Vendor ID".into(),
        "recipe_id" => "Recipe ID".into(),
        "recipe_group_id" => "Recipe Group".into(),
        "invoice_id" => "Invoice ID".into(),
        "trans_id" => "Transaction ID".into(),
        "recp_item_id" => "Recipe Item ID".into(),
        "recp_inv_id" => "Recipe Inv. ID".into(),
        "purch_unit_id" => "Purchase Unit ID".into(),
        "recipe_unit_id" => "Recipe Unit ID".into(),

        // ── units table ──
        "sing" => "Singular".into(),
        "plur" => "Plural".into(),
        "unit_type" => "Unit Type".into(),
        "is_whole_unit" => "Whole Unit".into(),
        "unit_kind" => "Kind".into(),

        // ── items ──
        "name" => "Name".into(),
        "item_name" => "Item Name".into(),
        "recipe_name" => "Recipe Name".into(),
        "status" => "Status".into(),
        "raw_len" => "Raw Length".into(),

        // ── convunit ──
        "unit_id1" => "From Unit".into(),
        "unit_id2" => "To Unit".into(),
        "qty1" => "Qty From".into(),
        "qty2" => "Qty To".into(),
        "qty" => "Quantity".into(),
        "is_calculated" => "Calculated".into(),

        // ── inv_units / inv_prices ──
        "is_default" => "Default".into(),
        "price" => "Price".into(),
        "pack" => "Pack".into(),

        // ── conv_suggestions / conv_todo ──
        "recipe_unit" => "Recipe Unit".into(),
        "purch_unit" => "Purchase Unit".into(),
        "hits" => "Hits".into(),
        "derived_from" => "Derived From".into(),
        "hops" => "Hops".into(),
        "path" => "Path".into(),
        "needed" => "Needed".into(),

        // ── missing_edges ──
        "usage_count" => "Usage Count".into(),

        // ── missing_data_report ──
        "missing_a" => "Missing Costs".into(),
        "missing_b" => "Missing Conversions".into(),
        "missing_c" => "Missing Units".into(),

        // ── invoices ──
        "invoice_date" | "trans_date" => "Date".into(),
        "invoice_no" => "Invoice #".into(),
        "freight" => "Freight".into(),
        "total" => "Total".into(),
        "ext_cost" => "Ext. Cost".into(),

        // ── generic colN placeholders ──
        other => {
            if let Some(rest) = other.strip_prefix("col") {
                if rest.chars().all(|c| c.is_ascii_digit()) {
                    return format!("Column {rest}");
                }
            }
            // Fallback: replace underscores and title-case
            other
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => {
                            let mut s = c.to_uppercase().to_string();
                            s.extend(chars);
                            s
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (base_path, set_base_path) = signal("~/exports".to_string());
    let (import_mode, set_import_mode) = signal("exports".to_string());
    let (summary, set_summary) = signal(ImportSummary::default());
    let (has_summary, set_has_summary) = signal(false);
    let (status, set_status) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (active_panel, set_active_panel) = signal("imports".to_string());
    let (show_warnings, set_show_warnings) = signal(false);

    let browse_limit: u32 = 100;
    let (browse_table_name, set_browse_table_name) = signal(String::new());
    let (browse_data, set_browse_data) = signal(BrowseTableResponse::default());
    let (browse_loading, set_browse_loading) = signal(false);
    let (browse_page, set_browse_page) = signal(0u32);

    let inventory_limit: usize = 50;
    let (inventory_query, set_inventory_query) = signal(String::new());
    let (inventory_items, set_inventory_items) = signal(Vec::<InventoryItem>::new());
    let (inventory_total, set_inventory_total) = signal(0i64);
    let (inventory_filtered, set_inventory_filtered) = signal(0i64);
    let (inventory_page, set_inventory_page) = signal(0usize);
    let (inventory_status, set_inventory_status) = signal(String::new());
    let (inventory_loading, set_inventory_loading) = signal(false);
    let (inventory_loaded, set_inventory_loaded) = signal(false);
    let (inventory_selected, set_inventory_selected) =
        signal(Option::<InventoryDetailResponse>::None);
    let (inventory_detail_status, set_inventory_detail_status) = signal(String::new());
    let (inventory_detail_loading, set_inventory_detail_loading) = signal(false);

    let recipe_limit: usize = 25;
    let (recipe_query, set_recipe_query) = signal(String::new());
    let (recipe_list, set_recipe_list) = signal(Vec::<RecipeListItem>::new());
    let (recipe_total, set_recipe_total) = signal(0i64);
    let (recipe_filtered, set_recipe_filtered) = signal(0i64);
    let (recipe_page, set_recipe_page) = signal(0usize);
    let (recipe_status, set_recipe_status) = signal(String::new());
    let (recipe_loading, set_recipe_loading) = signal(false);
    let (recipe_loaded, set_recipe_loaded) = signal(false);
    let (selected_recipe, set_selected_recipe) = signal(Option::<RecipeDetailResponse>::None);
    let (recipe_detail_status, set_recipe_detail_status) = signal(String::new());
    let (recipe_detail_loading, set_recipe_detail_loading) = signal(false);
    let (tauri_status, set_tauri_status) = signal(String::new());
    let (ping_status, set_ping_status) = signal(String::new());

    let vendor_limit: usize = 50;
    let (vendor_query, set_vendor_query) = signal(String::new());
    let (vendor_list, set_vendor_list) = signal(Vec::<VendorListItem>::new());
    let (vendor_total, set_vendor_total) = signal(0i64);
    let (vendor_filtered, set_vendor_filtered) = signal(0i64);
    let (vendor_page, set_vendor_page) = signal(0usize);
    let (vendor_status, set_vendor_status) = signal(String::new());
    let (vendor_loading, set_vendor_loading) = signal(false);
    let (vendor_loaded, set_vendor_loaded) = signal(false);
    let (selected_vendor, set_selected_vendor) = signal(Option::<VendorDetailResponse>::None);
    let (vendor_detail_status, set_vendor_detail_status) = signal(String::new());
    let (vendor_detail_loading, set_vendor_detail_loading) = signal(false);
    let (vendor_merge_options, set_vendor_merge_options) = signal(Vec::<VendorSimple>::new());
    let (vendor_merge_source_id, set_vendor_merge_source_id) = signal(String::new());
    let (vendor_merge_target_id, set_vendor_merge_target_id) = signal(String::new());
    let (vendor_merge_status, set_vendor_merge_status) = signal(String::new());

    let conversion_limit: usize = 50;
    let (conversion_tab, set_conversion_tab) = signal("suggestions".to_string());
    let (conversion_page, set_conversion_page) = signal(0usize);
    let (conversion_status, set_conversion_status) = signal(String::new());
    let (conversion_loading, set_conversion_loading) = signal(false);
    let (conversion_overview, set_conversion_overview) = signal(ConversionOverview::default());
    let (conversion_suggestions, set_conversion_suggestions) =
        signal(Vec::<ConversionSuggestionRow>::new());
    let (conversion_suggestions_total, set_conversion_suggestions_total) = signal(0i64);
    let (conversion_suggestions_safe, set_conversion_suggestions_safe) =
        signal(Vec::<ConversionSuggestionRow>::new());
    let (conversion_suggestions_safe_total, set_conversion_suggestions_safe_total) = signal(0i64);
    let (conversion_todo, set_conversion_todo) = signal(Vec::<ConversionTodoRow>::new());
    let (conversion_todo_total, set_conversion_todo_total) = signal(0i64);
    let (conversion_missing_edges, set_conversion_missing_edges) =
        signal(Vec::<MissingEdgeRow>::new());
    let (conversion_missing_edges_total, set_conversion_missing_edges_total) = signal(0i64);

    let report_limit: usize = 50;
    let (report_page, _set_report_page) = signal(0usize);
    let (report_status, set_report_status) = signal(String::new());
    let (_report_loading, set_report_loading) = signal(false);
    let (missing_data_rows, set_missing_data_rows) = signal(Vec::<MissingDataRow>::new());
    let (missing_data_total, set_missing_data_total) = signal(0i64);
    let (missing_purch_rows, set_missing_purch_rows) = signal(Vec::<MissingPurchRow>::new());
    let (missing_purch_total, set_missing_purch_total) = signal(0i64);
    let (report_purch_item_id, set_report_purch_item_id) = signal(String::new());

    let invoice_limit: usize = 50;
    let (invoice_query, set_invoice_query) = signal(String::new());
    let (invoice_list, set_invoice_list) = signal(Vec::<InvoiceListItem>::new());
    let (invoice_total, set_invoice_total) = signal(0i64);
    let (invoice_filtered, set_invoice_filtered) = signal(0i64);
    let (invoice_page, set_invoice_page) = signal(0usize);
    let (invoice_status, set_invoice_status) = signal(String::new());
    let (invoice_loading, set_invoice_loading) = signal(false);
    let (invoice_loaded, set_invoice_loaded) = signal(false);
    let (selected_invoice, set_selected_invoice) = signal(Option::<InvoiceDetailResponse>::None);
    let (invoice_detail_status, set_invoice_detail_status) = signal(String::new());
    let (invoice_detail_loading, set_invoice_detail_loading) = signal(false);
    let (invoice_vendor_filter, set_invoice_vendor_filter) = signal(String::new());
    let (invoice_vendor_options, set_invoice_vendor_options) = signal(Vec::<VendorSimple>::new());
    let (editor_vendor_options, set_editor_vendor_options) = signal(Vec::<VendorSimple>::new());
    let (unit_options, set_unit_options) = signal(Vec::<UnitSimple>::new());
    let (invoice_date_from, set_invoice_date_from) = signal(String::new());
    let (invoice_date_to, set_invoice_date_to) = signal(String::new());
    let (invoice_export_status, set_invoice_export_status) = signal(String::new());
    let (invoice_export_path, set_invoice_export_path) = signal(String::new());
    let (purch_unit_selected_id, set_purch_unit_selected_id) = signal(String::new());
    let (purch_unit_set_default, set_purch_unit_set_default) = signal(true);
    let (purch_unit_status, set_purch_unit_status) = signal(String::new());
    let (manual_price_vendor_id, set_manual_price_vendor_id) = signal(String::new());
    let (manual_price_value, set_manual_price_value) = signal(String::new());
    let (manual_price_pack, set_manual_price_pack) = signal(String::new());
    let (manual_price_status, set_manual_price_status) = signal(String::new());

    let start_import = move || {
        let path = base_path.get();
        let mode = import_mode.get();
        set_busy.set(true);
        set_status.set(if mode == "mdf" {
            "Importing from MDF/LDF...".to_string()
        } else {
            "Importing exports...".to_string()
        });

        spawn_local(async move {
            let result = if mode == "mdf" {
                let args = to_value(&ImportMdfArgs { mdf_path: path }).unwrap();
                invoke_cmd::<ImportSummary>("import_from_mdf", args).await
            } else {
                let args = to_value(&ImportArgs { base_path: path }).unwrap();
                invoke_cmd::<ImportSummary>("import_exports", args).await
            };

            match result {
                Ok(result) => {
                    set_summary.set(result);
                    set_has_summary.set(true);
                    set_status.set("Import complete".to_string());
                    set_inventory_loaded.set(false);
                    set_recipe_loaded.set(false);
                    set_selected_recipe.set(None);
                    set_vendor_loaded.set(false);
                    set_invoice_loaded.set(false);
                }
                Err(err) => {
                    set_status.set(format!("Import failed: {err}"));
                }
            }
            set_busy.set(false);
        });
    };

    Effect::new(move |_| {
        let status = match window() {
            Some(win) => {
                if let Ok(tauri_obj) = Reflect::get(&win, &JsValue::from_str("__TAURI__")) {
                    if tauri_obj.is_undefined() || tauri_obj.is_null() {
                        "Tauri API missing".to_string()
                    } else if let Ok(core) = Reflect::get(&tauri_obj, &JsValue::from_str("core")) {
                        if core.is_undefined() || core.is_null() {
                            "Tauri core missing".to_string()
                        } else if let Ok(invoke_fn) =
                            Reflect::get(&core, &JsValue::from_str("invoke"))
                        {
                            if invoke_fn.is_function() {
                                "Tauri API ready".to_string()
                            } else {
                                "Tauri invoke missing".to_string()
                            }
                        } else {
                            "Tauri core missing".to_string()
                        }
                    } else {
                        "Tauri API missing".to_string()
                    }
                } else {
                    "Tauri API missing".to_string()
                }
            }
            None => "Window missing".to_string(),
        };
        set_tauri_status.set(status);
    });

    let ping_backend = move || {
        set_ping_status.set("Pinging backend...".to_string());
        spawn_local(async move {
            let args = to_value(&PingArgs {}).unwrap();
            match invoke_cmd::<PingResponse>("ping_backend", args).await {
                Ok(resp) => {
                    set_ping_status.set(format!("Ping: {} ({})", resp.message, resp.ok));
                }
                Err(err) => {
                    set_ping_status.set(format!("Ping failed: {err}"));
                }
            }
        });
    };

    let open_path = move |path: String| {
        spawn_local(async move {
            let args = to_value(&OpenPathArgs { path }).unwrap();
            let _ = invoke_cmd::<PatchResponse>("open_path", args).await;
        });
    };

    let fetch_browse_table = move |table: String, page: u32| {
        set_browse_loading.set(true);
        spawn_local(async move {
            let args = to_value(&BrowseTableArgs {
                table_name: table,
                limit: browse_limit,
                offset: page * browse_limit,
            })
            .unwrap();
            match invoke_cmd::<BrowseTableResponse>("browse_table", args).await {
                Ok(resp) => {
                    set_browse_data.set(resp);
                }
                Err(_) => {
                    set_browse_data.set(BrowseTableResponse::default());
                }
            }
            set_browse_loading.set(false);
        });
    };

    let toggle_card = move |table: &str| {
        let table = table.to_string();
        if browse_table_name.get() == table {
            set_browse_table_name.set(String::new());
            set_browse_data.set(BrowseTableResponse::default());
            set_browse_page.set(0);
        } else {
            set_browse_table_name.set(table.clone());
            set_browse_page.set(0);
            fetch_browse_table(table, 0);
        }
    };

    let show_imports = move || {
        set_active_panel.set("imports".to_string());
    };

    let show_inventory = move || {
        set_active_panel.set("inventory".to_string());
        if editor_vendor_options.get().is_empty() {
            trigger_vendor_options_fetch(set_editor_vendor_options);
        }
        if unit_options.get().is_empty() {
            trigger_unit_options_fetch(set_unit_options);
        }
        if !inventory_loaded.get() {
            trigger_inventory_fetch(
                inventory_query.get(),
                inventory_page.get(),
                inventory_limit,
                set_inventory_loading,
                set_inventory_status,
                set_inventory_items,
                set_inventory_total,
                set_inventory_filtered,
                set_inventory_loaded,
            );
        }
    };

    let show_recipes = move || {
        set_active_panel.set("recipes".to_string());
        if !recipe_loaded.get() {
            trigger_recipe_fetch(
                recipe_query.get(),
                recipe_page.get(),
                recipe_limit,
                set_recipe_loading,
                set_recipe_status,
                set_recipe_list,
                set_recipe_total,
                set_recipe_filtered,
                set_recipe_loaded,
                selected_recipe.get().is_none(),
                set_recipe_detail_loading,
                set_recipe_detail_status,
                set_selected_recipe,
            );
        }
    };

    let show_vendors = move || {
        set_active_panel.set("vendors".to_string());
        if vendor_merge_options.get().is_empty() {
            trigger_vendor_all_options_fetch(set_vendor_merge_options);
        }
        if !vendor_loaded.get() {
            trigger_vendor_fetch(
                vendor_query.get(),
                vendor_page.get(),
                vendor_limit,
                set_vendor_loading,
                set_vendor_status,
                set_vendor_list,
                set_vendor_total,
                set_vendor_filtered,
                set_vendor_loaded,
            );
        }
    };

    let show_conversions = move || {
        set_active_panel.set("conversions".to_string());
        trigger_conversion_overview_fetch(set_conversion_overview);
        let tab = conversion_tab.get();
        let page = conversion_page.get();
        match tab.as_str() {
            "suggestions" => trigger_conversion_suggestions_fetch(
                "conv_suggestions".to_string(),
                page,
                conversion_limit,
                set_conversion_loading,
                set_conversion_status,
                set_conversion_suggestions,
                set_conversion_suggestions_total,
            ),
            "safe" => trigger_conversion_suggestions_fetch(
                "conv_suggestions_safe".to_string(),
                page,
                conversion_limit,
                set_conversion_loading,
                set_conversion_status,
                set_conversion_suggestions_safe,
                set_conversion_suggestions_safe_total,
            ),
            "todo" => trigger_conversion_todo_fetch(
                page,
                conversion_limit,
                set_conversion_loading,
                set_conversion_status,
                set_conversion_todo,
                set_conversion_todo_total,
            ),
            "missing" => trigger_missing_edges_fetch(
                page,
                conversion_limit,
                set_conversion_loading,
                set_conversion_status,
                set_conversion_missing_edges,
                set_conversion_missing_edges_total,
            ),
            _ => {}
        }
    };

    let show_reports = move || {
        set_active_panel.set("reports".to_string());
        if unit_options.get().is_empty() {
            trigger_unit_options_fetch(set_unit_options);
        }
        trigger_missing_data_fetch(
            report_page.get(),
            report_limit,
            set_report_loading,
            set_report_status,
            set_missing_data_rows,
            set_missing_data_total,
        );
        trigger_missing_purch_fetch(
            report_page.get(),
            report_limit,
            set_report_loading,
            set_report_status,
            set_missing_purch_rows,
            set_missing_purch_total,
        );
    };

    let show_purchasing = move || {
        set_active_panel.set("purchasing".to_string());
        if invoice_vendor_options.get().is_empty() {
            trigger_vendor_options_fetch(set_invoice_vendor_options);
        }
        if !invoice_loaded.get() {
            trigger_invoice_fetch(
                invoice_query.get(),
                invoice_vendor_filter.get().trim().parse::<i64>().ok(),
                invoice_date_from.get(),
                invoice_date_to.get(),
                invoice_page.get(),
                invoice_limit,
                set_invoice_loading,
                set_invoice_status,
                set_invoice_list,
                set_invoice_total,
                set_invoice_filtered,
                set_invoice_loaded,
            );
        }
    };

    // ── Food Cost state ──
    let (fc_item_options, set_fc_item_options) = signal(Vec::<ItemSimple>::new());
    let (fc_unit_options, set_fc_unit_options) = signal(Vec::<UnitSimple>::new());
    let (fc_lines, set_fc_lines) = signal(Vec::<FoodCostLineInput>::new());
    let (fc_result, set_fc_result) = signal(Option::<FoodCostResponse>::None);
    let (fc_target_pct, set_fc_target_pct) = signal(30.0f64);
    let (fc_status, set_fc_status) = signal(String::new());
    let (fc_loading, set_fc_loading) = signal(false);
    let (fc_add_item_id, set_fc_add_item_id) = signal(String::new());
    let (fc_add_unit_id, set_fc_add_unit_id) = signal(String::new());
    let (fc_add_qty, set_fc_add_qty) = signal(String::new());
    let (fc_dish_name, set_fc_dish_name) = signal(String::new());

    // ── Settings state ──
    let (settings_company, set_settings_company) = signal(String::new());
    let (settings_logo_path, set_settings_logo_path) = signal(String::new());
    let (settings_service_cat, set_settings_service_cat) = signal(String::new());
    let (settings_op_size, set_settings_op_size) = signal(String::new());
    let (settings_status, set_settings_status) = signal(String::new());
    let (settings_loaded, set_settings_loaded) = signal(false);
    let (settings_logo_upload_path, set_settings_logo_upload_path) = signal(String::new());

    let load_settings = move || {
        spawn_local(async move {
            let args = to_value(&PingArgs {}).unwrap();
            match invoke_cmd::<SettingsResponse>("get_settings", args).await {
                Ok(s) => {
                    set_settings_company.set(s.company_name);
                    set_settings_logo_path.set(s.logo_path);
                    set_settings_service_cat.set(s.service_category);
                    set_settings_op_size.set(s.operation_size);
                    set_settings_loaded.set(true);
                }
                Err(err) => {
                    set_settings_status.set(format!("Failed to load settings: {err}"));
                }
            }
        });
    };

    let show_settings = move || {
        set_active_panel.set("settings".to_string());
        if !settings_loaded.get() {
            load_settings();
        }
    };

    let show_fda = move || {
        set_active_panel.set("fda".to_string());
    };

    let save_settings_action = move || {
        let company_name = settings_company.get();
        let service_category = settings_service_cat.get();
        let operation_size = settings_op_size.get();
        set_settings_status.set("Saving...".to_string());
        spawn_local(async move {
            let args = to_value(&SaveSettingsArgs {
                company_name,
                service_category,
                operation_size,
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("save_settings", args).await {
                Ok(resp) => set_settings_status.set(resp.message),
                Err(err) => set_settings_status.set(format!("Save failed: {err}")),
            }
        });
    };

    let upload_logo_action = move || {
        let source = settings_logo_upload_path.get();
        if source.trim().is_empty() {
            set_settings_status.set("Enter a logo file path".to_string());
            return;
        }
        set_settings_status.set("Uploading logo...".to_string());
        spawn_local(async move {
            let args = to_value(&UploadLogoArgs { source_path: source }).unwrap();
            match invoke_cmd::<PatchResponse>("upload_logo", args).await {
                Ok(resp) => {
                    set_settings_status.set(resp.message);
                    // reload to get the new path
                    let args2 = to_value(&PingArgs {}).unwrap();
                    if let Ok(s) = invoke_cmd::<SettingsResponse>("get_settings", args2).await {
                        set_settings_logo_path.set(s.logo_path);
                    }
                    set_settings_logo_upload_path.set(String::new());
                }
                Err(err) => set_settings_status.set(format!("Upload failed: {err}")),
            }
        });
    };

    let remove_logo_action = move || {
        set_settings_status.set("Removing logo...".to_string());
        spawn_local(async move {
            let args = to_value(&PingArgs {}).unwrap();
            match invoke_cmd::<PatchResponse>("remove_logo", args).await {
                Ok(resp) => {
                    set_settings_status.set(resp.message);
                    set_settings_logo_path.set(String::new());
                }
                Err(err) => set_settings_status.set(format!("Remove failed: {err}")),
            }
        });
    };

    let show_foodcost = move || {
        set_active_panel.set("foodcost".to_string());
        if fc_item_options.get().is_empty() {
            spawn_local(async move {
                let args = to_value(&PingArgs {}).unwrap();
                if let Ok(result) = invoke_cmd::<ItemSimpleResponse>("list_items_simple", args).await {
                    set_fc_item_options.set(result.items);
                }
            });
        }
        if fc_unit_options.get().is_empty() {
            trigger_unit_options_fetch(set_fc_unit_options);
        }
    };

    let fc_add_line = move || {
        let item_id = match fc_add_item_id.get().trim().parse::<i64>() {
            Ok(v) => v,
            Err(_) => {
                set_fc_status.set("Select an ingredient".to_string());
                return;
            }
        };
        let unit_id: Option<i64> = fc_add_unit_id.get().trim().parse().ok();
        let qty: f64 = fc_add_qty.get().trim().parse().unwrap_or(0.0);
        if qty <= 0.0 {
            set_fc_status.set("Enter a quantity > 0".to_string());
            return;
        }
        let mut current = fc_lines.get();
        current.push(FoodCostLineInput { item_id, unit_id, qty });
        set_fc_lines.set(current);
        set_fc_add_item_id.set(String::new());
        set_fc_add_unit_id.set(String::new());
        set_fc_add_qty.set(String::new());
        set_fc_status.set(String::new());
    };

    let fc_remove_line = move |idx: usize| {
        let mut current = fc_lines.get();
        if idx < current.len() {
            current.remove(idx);
            set_fc_lines.set(current);
            set_fc_result.set(None);
        }
    };

    let fc_calculate = move || {
        let lines = fc_lines.get();
        if lines.is_empty() {
            set_fc_status.set("Add at least one ingredient".to_string());
            return;
        }
        set_fc_loading.set(true);
        set_fc_status.set("Calculating...".to_string());
        spawn_local(async move {
            let args = to_value(&CalculateFoodCostArgs { lines }).unwrap();
            match invoke_cmd::<FoodCostResponse>("calculate_food_cost", args).await {
                Ok(result) => {
                    set_fc_result.set(Some(result));
                    set_fc_status.set("Done".to_string());
                }
                Err(err) => {
                    set_fc_status.set(format!("Calculation failed: {err}"));
                }
            }
            set_fc_loading.set(false);
        });
    };

    let fc_clear = move || {
        set_fc_lines.set(Vec::new());
        set_fc_result.set(None);
        set_fc_status.set(String::new());
        set_fc_dish_name.set(String::new());
    };

    let export_invoices = move || {
        let query = invoice_query.get();
        let vendor_id = invoice_vendor_filter.get().trim().parse::<i64>().ok();
        let date_from = invoice_date_from.get();
        let date_to = invoice_date_to.get();
        set_invoice_export_status.set("Exporting...".to_string());
        spawn_local(async move {
            let args = to_value(&ExportInvoiceArgs {
                query,
                vendor_id,
                date_from,
                date_to,
                output_path: None,
            })
            .unwrap();
            match invoke_cmd::<ExportCsvResponse>("export_invoice_lines_csv", args).await {
                Ok(result) => {
                    set_invoice_export_path.set(result.path.clone());
                    set_invoice_export_status.set(format!("Exported {} rows", result.rows));
                }
                Err(err) => {
                    set_invoice_export_status.set(format!("Export failed: {err}"));
                }
            }
        });
    };

    let approve_conversion = move |row: ConversionSuggestionRow| {
        let base_path = base_path.get();
        set_conversion_status.set("Saving patch...".to_string());
        spawn_local(async move {
            let args = to_value(&PatchConvunitArgs {
                base_path,
                item_id: row.item_id,
                vendor_id: row.vendor_id,
                unit_id1: row.unit_id1,
                unit_id2: row.unit_id2,
                qty1: row.qty1,
                qty2: row.qty2,
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("patch_convunit", args).await {
                Ok(resp) => {
                    set_conversion_status.set(resp.message);
                    trigger_conversion_overview_fetch(set_conversion_overview);
                    let table = conversion_tab.get();
                    let page = conversion_page.get();
                    match table.as_str() {
                        "suggestions" => trigger_conversion_suggestions_fetch(
                            "conv_suggestions".to_string(),
                            page,
                            conversion_limit,
                            set_conversion_loading,
                            set_conversion_status,
                            set_conversion_suggestions,
                            set_conversion_suggestions_total,
                        ),
                        "safe" => trigger_conversion_suggestions_fetch(
                            "conv_suggestions_safe".to_string(),
                            page,
                            conversion_limit,
                            set_conversion_loading,
                            set_conversion_status,
                            set_conversion_suggestions_safe,
                            set_conversion_suggestions_safe_total,
                        ),
                        "todo" => trigger_conversion_todo_fetch(
                            page,
                            conversion_limit,
                            set_conversion_loading,
                            set_conversion_status,
                            set_conversion_todo,
                            set_conversion_todo_total,
                        ),
                        "missing" => trigger_missing_edges_fetch(
                            page,
                            conversion_limit,
                            set_conversion_loading,
                            set_conversion_status,
                            set_conversion_missing_edges,
                            set_conversion_missing_edges_total,
                        ),
                        _ => {}
                    }
                }
                Err(err) => set_conversion_status.set(format!("Patch failed: {err}")),
            }
        });
    };

    let assign_purch_unit = move |item_id: i64| {
        let purch_unit_id = match purch_unit_selected_id.get().trim().parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                set_purch_unit_status.set("Enter a valid purchase unit".to_string());
                return;
            }
        };
        let is_default = purch_unit_set_default.get();
        set_purch_unit_status.set("Saving purchase unit...".to_string());
        spawn_local(async move {
            let args = to_value(&SetPurchUnitArgs {
                item_id,
                purch_unit_id,
                is_default,
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("set_item_purch_unit", args).await {
                Ok(resp) => {
                    set_purch_unit_status.set(resp.message);
                    if active_panel.get() == "reports" {
                        trigger_missing_purch_fetch(
                            report_page.get(),
                            report_limit,
                            set_report_loading,
                            set_report_status,
                            set_missing_purch_rows,
                            set_missing_purch_total,
                        );
                    }
                    if let Some(detail) = inventory_selected.get() {
                        if detail.item_id == item_id {
                            trigger_inventory_detail_fetch(
                                item_id,
                                set_inventory_detail_loading,
                                set_inventory_detail_status,
                                set_inventory_selected,
                            );
                        }
                    }
                }
                Err(err) => set_purch_unit_status.set(format!("Save failed: {err}")),
            }
        });
    };

    let save_manual_price = move |item_id: i64| {
        let vendor_id = match manual_price_vendor_id.get().trim().parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                set_manual_price_status.set("Select a vendor".to_string());
                return;
            }
        };
        let price = match manual_price_value.get().trim().parse::<f64>() {
            Ok(value) if value > 0.0 => value,
            _ => {
                set_manual_price_status.set("Enter a valid price > 0".to_string());
                return;
            }
        };
        let pack = manual_price_pack.get();
        set_manual_price_status.set("Saving manual price...".to_string());
        spawn_local(async move {
            let args = to_value(&UpsertManualPriceArgs {
                item_id,
                vendor_id,
                price,
                pack,
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("upsert_manual_price", args).await {
                Ok(resp) => {
                    set_manual_price_status.set(resp.message);
                    trigger_inventory_detail_fetch(
                        item_id,
                        set_inventory_detail_loading,
                        set_inventory_detail_status,
                        set_inventory_selected,
                    );
                }
                Err(err) => set_manual_price_status.set(format!("Save failed: {err}")),
            }
        });
    };

    let merge_vendor_ids = move || {
        let source_vendor_id = match vendor_merge_source_id.get().trim().parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                set_vendor_merge_status.set("Enter a valid source vendor ID".to_string());
                return;
            }
        };
        let target_vendor_id = match vendor_merge_target_id.get().trim().parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                set_vendor_merge_status.set("Select a valid target vendor".to_string());
                return;
            }
        };
        if source_vendor_id == target_vendor_id {
            set_vendor_merge_status.set("Source and target vendors must be different".to_string());
            return;
        }
        set_vendor_merge_status.set("Merging vendors...".to_string());
        spawn_local(async move {
            let args = to_value(&MergeVendorArgs {
                source_vendor_id,
                target_vendor_id,
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("merge_vendor", args).await {
                Ok(resp) => {
                    set_vendor_merge_status.set(resp.message);
                    set_vendor_merge_source_id.set(target_vendor_id.to_string());
                    trigger_vendor_fetch(
                        vendor_query.get(),
                        vendor_page.get(),
                        vendor_limit,
                        set_vendor_loading,
                        set_vendor_status,
                        set_vendor_list,
                        set_vendor_total,
                        set_vendor_filtered,
                        set_vendor_loaded,
                    );
                    trigger_vendor_detail_fetch(
                        target_vendor_id,
                        set_vendor_detail_loading,
                        set_vendor_detail_status,
                        set_selected_vendor,
                    );
                    trigger_vendor_options_fetch(set_invoice_vendor_options);
                    trigger_vendor_options_fetch(set_editor_vendor_options);
                    trigger_vendor_all_options_fetch(set_vendor_merge_options);
                }
                Err(err) => set_vendor_merge_status.set(format!("Merge failed: {err}")),
            }
        });
    };

    // ── Inventory edit signals ──
    let (edit_item_name, set_edit_item_name) = signal(String::new());
    let (edit_item_status, set_edit_item_status) = signal(String::new());
    let (edit_item_msg, set_edit_item_msg) = signal(String::new());

    let save_item_edit = move |item_id: i64| {
        let name = edit_item_name.get();
        let status_val: Option<i64> = edit_item_status.get().parse().ok();
        set_edit_item_msg.set("Saving...".to_string());
        spawn_local(async move {
            let args = to_value(&UpdateItemArgs {
                item_id,
                name,
                status: status_val,
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("update_item", args).await {
                Ok(resp) => {
                    set_edit_item_msg.set(resp.message);
                    // Refresh detail
                    trigger_inventory_detail_fetch(
                        item_id,
                        set_inventory_detail_loading,
                        set_inventory_detail_status,
                        set_inventory_selected,
                    );
                    // Refresh list
                    trigger_inventory_fetch(
                        inventory_query.get(),
                        inventory_page.get(),
                        inventory_limit,
                        set_inventory_loading,
                        set_inventory_status,
                        set_inventory_items,
                        set_inventory_total,
                        set_inventory_filtered,
                        set_inventory_loaded,
                    );
                }
                Err(err) => set_edit_item_msg.set(format!("Save failed: {err}")),
            }
        });
    };

    // ── Recipe edit signals ──
    let (edit_recipe_name, set_edit_recipe_name) = signal(String::new());
    let (edit_recipe_instructions, set_edit_recipe_instructions) = signal(String::new());
    let (edit_recipe_msg, set_edit_recipe_msg) = signal(String::new());
    let (edit_recipe_editing, set_edit_recipe_editing) = signal(false);

    let save_recipe_edit = move |recipe_id: i64| {
        let name = edit_recipe_name.get();
        let instructions = edit_recipe_instructions.get();
        set_edit_recipe_msg.set("Saving...".to_string());
        spawn_local(async move {
            let args = to_value(&UpdateRecipeArgs {
                recipe_id,
                name,
                instructions,
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("update_recipe", args).await {
                Ok(resp) => {
                    set_edit_recipe_msg.set(resp.message);
                    set_edit_recipe_editing.set(false);
                    trigger_recipe_detail_fetch(
                        recipe_id,
                        set_recipe_detail_loading,
                        set_recipe_detail_status,
                        set_selected_recipe,
                    );
                    trigger_recipe_fetch(
                        recipe_query.get(),
                        recipe_page.get(),
                        recipe_limit,
                        set_recipe_loading,
                        set_recipe_status,
                        set_recipe_list,
                        set_recipe_total,
                        set_recipe_filtered,
                        set_recipe_loaded,
                        false,
                        set_recipe_detail_loading,
                        set_recipe_detail_status,
                        set_selected_recipe,
                    );
                }
                Err(err) => set_edit_recipe_msg.set(format!("Save failed: {err}")),
            }
        });
    };

    let delete_recipe_ingredient = move |recipe_id: i64, recp_item_id: i64| {
        set_edit_recipe_msg.set("Removing ingredient...".to_string());
        spawn_local(async move {
            let args = to_value(&AddRecpItemArgs {
                recipe_id,
                recp_item_id: Some(recp_item_id),
                item_id: 0,
                unit_id: None,
                qty: Some(0.0),
            })
            .unwrap();
            match invoke_cmd::<PatchResponse>("add_recp_item", args).await {
                Ok(resp) => {
                    set_edit_recipe_msg.set(resp.message);
                    trigger_recipe_detail_fetch(
                        recipe_id,
                        set_recipe_detail_loading,
                        set_recipe_detail_status,
                        set_selected_recipe,
                    );
                }
                Err(err) => set_edit_recipe_msg.set(format!("Remove failed: {err}")),
            }
        });
    };

    // ── Export signals ──
    let (export_status, set_export_status) = signal(String::new());

    /// Open a native Save dialog, then invoke an export command.
    fn trigger_save_dialog_and_export(
        title: &str,
        default_name: &str,
        filter_label: &str,
        filter_ext: &str,
        set_export_status: WriteSignal<String>,
        build_cmd: impl FnOnce(String) -> (String, JsValue) + 'static,
    ) {
        let title = title.to_string();
        let default_name = default_name.to_string();
        let filter_label = filter_label.to_string();
        let filter_ext = filter_ext.to_string();
        spawn_local(async move {
            let path: Option<String> = {
                let w = match window() {
                    Some(w) => w,
                    None => { set_export_status.set("Window not available".into()); return; }
                };
                let tauri = match Reflect::get(&w, &JsValue::from_str("__TAURI__")) {
                    Ok(t) if !t.is_undefined() && !t.is_null() => t,
                    _ => { set_export_status.set("Tauri API missing".into()); return; }
                };
                let dialog = match Reflect::get(&tauri, &JsValue::from_str("dialog")) {
                    Ok(d) if !d.is_undefined() && !d.is_null() => d,
                    _ => { set_export_status.set("Dialog API missing".into()); return; }
                };
                let save_fn = match Reflect::get(&dialog, &JsValue::from_str("save")) {
                    Ok(f) if f.is_function() => f,
                    _ => { set_export_status.set("Save dialog missing".into()); return; }
                };
                let opts = js_sys::Object::new();
                let _ = Reflect::set(&opts, &JsValue::from_str("title"), &JsValue::from_str(&title));
                let _ = Reflect::set(&opts, &JsValue::from_str("defaultPath"), &JsValue::from_str(&default_name));
                // Filter
                let filter_obj = js_sys::Object::new();
                let _ = Reflect::set(&filter_obj, &JsValue::from_str("name"), &JsValue::from_str(&filter_label));
                let exts = js_sys::Array::new();
                exts.push(&JsValue::from_str(&filter_ext));
                let _ = Reflect::set(&filter_obj, &JsValue::from_str("extensions"), &exts);
                let filters = js_sys::Array::new();
                filters.push(&filter_obj);
                let _ = Reflect::set(&opts, &JsValue::from_str("filters"), &filters);

                let func = save_fn.dyn_ref::<js_sys::Function>().unwrap();
                match func.call1(&JsValue::NULL, &opts) {
                    Ok(promise) => {
                        if let Ok(p) = promise.dyn_into::<Promise>() {
                            match JsFuture::from(p).await {
                                Ok(result) => result.as_string(),
                                Err(_) => None,
                            }
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            };

            if let Some(output_path) = path {
                set_export_status.set("Exporting...".to_string());
                let (cmd, args) = build_cmd(output_path);
                match invoke_cmd::<PatchResponse>(&cmd, args).await {
                    Ok(resp) => set_export_status.set(resp.message),
                    Err(err) => set_export_status.set(format!("Export failed: {err}")),
                }
            }
        });
    }

    let export_inventory_pdf = move || {
        trigger_save_dialog_and_export(
            "Save Inventory PDF",
            "inventory.pdf",
            "PDF",
            "pdf",
            set_export_status,
            |path| ("export_inventory_pdf".to_string(), to_value(&ExportPathArgs { output_path: path }).unwrap()),
        );
    };

    let export_inventory_docx = move || {
        trigger_save_dialog_and_export(
            "Save Inventory Word Document",
            "inventory.docx",
            "Word Document",
            "docx",
            set_export_status,
            |path| ("export_inventory_docx".to_string(), to_value(&ExportPathArgs { output_path: path }).unwrap()),
        );
    };

    // ── FDA PDF export ──
    let export_fda_pdf = move || {
        trigger_save_dialog_and_export(
            "Save FDA Guidelines PDF",
            "fda-guidelines.pdf",
            "PDF",
            "pdf",
            set_export_status,
            |path| ("export_fda_pdf".to_string(), to_value(&ExportPathArgs { output_path: path }).unwrap()),
        );
    };

    // ── Global search signals ──
    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal(Vec::<GlobalSearchHit>::new());
    let (_search_loading, set_search_loading) = signal(false);

    let do_global_search = move || {
        let q = search_query.get();
        if q.trim().is_empty() {
            set_search_results.set(Vec::new());
            return;
        }
        set_search_loading.set(true);
        spawn_local(async move {
            let args = to_value(&GlobalSearchArgs { query: q }).unwrap();
            match JsFuture::from(invoke("global_search", args)).await {
                Ok(val) => {
                    if let Ok(hits) = serde_wasm_bindgen::from_value::<Vec<GlobalSearchHit>>(val) {
                        set_search_results.set(hits);
                    }
                }
                Err(_) => {
                    set_search_results.set(Vec::new());
                }
            }
            set_search_loading.set(false);
        });
    };

    // ── PDF Invoice import signals ──
    let (pdf_preview, set_pdf_preview) = signal(Option::<PdfInvoicePreview>::None);
    let (pdf_import_status, set_pdf_import_status) = signal(String::new());
    let (pdf_vendor_id, set_pdf_vendor_id) = signal(String::new());
    let (pdf_invoice_no, set_pdf_invoice_no) = signal(String::new());
    let (pdf_invoice_date, set_pdf_invoice_date) = signal(String::new());

    view! {
        <div class="app-shell">
            <aside class="sidebar">
                <div class="brand">
                    <img src="public/4chef-logo.png" alt="4chef" class="brand-logo" />
                </div>
                <div class="sidebar-search">
                    <input
                        type="text"
                        class="sidebar-search-input"
                        placeholder="Search everything..."
                        prop:value=move || search_query.get()
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            set_search_query.set(val);
                            do_global_search();
                        }
                        on:keydown=move |ev| {
                            if ev.key() == "Escape" {
                                set_search_query.set(String::new());
                                set_search_results.set(Vec::new());
                            }
                        }
                    />
                    <Show when=move || !search_results.get().is_empty()>
                        <div class="sidebar-search-results">
                            <For
                                each=move || search_results.get()
                                key=|hit| format!("{}-{}", hit.category, hit.id)
                                children=move |hit: GlobalSearchHit| {
                                    let cat = hit.category.clone();
                                    let label = hit.label.clone();
                                    let detail = hit.detail.clone();
                                    view! {
                                        <button
                                            class="sidebar-search-hit"
                                            type="button"
                                            on:click=move |_| {
                                                match cat.as_str() {
                                                    "item" => set_active_panel.set("inventory".to_string()),
                                                    "recipe" => set_active_panel.set("recipes".to_string()),
                                                    "vendor" => set_active_panel.set("vendors".to_string()),
                                                    "invoice" => set_active_panel.set("purchasing".to_string()),
                                                    _ => {}
                                                }
                                                set_search_query.set(String::new());
                                                set_search_results.set(Vec::new());
                                            }
                                        >
                                            <span class="search-hit-label">{label.clone()}</span>
                                            <span class="search-hit-badge">{detail.clone()}</span>
                                        </button>
                                    }
                                }
                            />
                        </div>
                    </Show>
                </div>
                <div class="nav-section">
                    <div class="nav-label">"Modules"</div>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "imports"
                        on:click=move |_| show_imports()
                        type="button"
                    >
                        "Imports"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "inventory"
                        on:click=move |_| show_inventory()
                        type="button"
                    >
                        "Inventory"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "recipes"
                        on:click=move |_| show_recipes()
                        type="button"
                    >
                        "Recipes"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "vendors"
                        on:click=move |_| show_vendors()
                        type="button"
                    >
                        "Vendors"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "conversions"
                        on:click=move |_| show_conversions()
                        type="button"
                    >
                        "Conversions"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "purchasing"
                        on:click=move |_| show_purchasing()
                        type="button"
                    >
                        "Purchasing"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "reports"
                        on:click=move |_| show_reports()
                        type="button"
                    >
                        "Reports"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "foodcost"
                        on:click=move |_| show_foodcost()
                        type="button"
                    >
                        "Food Cost"
                    </button>
                </div>
                <div class="nav-section">
                    <div class="nav-label">"System"</div>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "settings"
                        on:click=move |_| show_settings()
                        type="button"
                    >
                        "Settings"
                    </button>
                    <button
                        class="nav-item"
                        class:active=move || active_panel.get() == "fda"
                        on:click=move |_| show_fda()
                        type="button"
                    >
                        "FDA Guidelines"
                    </button>
                </div>
            </aside>
            <main class="main">
                <div class="topbar">
                    <div>
                        <div class={move || {
                            if active_panel.get().as_str() == "imports" || active_panel.get().as_str() == "" || active_panel.get().as_str() == "home" {
                                "title title-home"
                            } else {
                                "title"
                            }
                        }}>
                            {move || {
                                match active_panel.get().as_str() {
                                    "inventory" => "Inventory Browser".to_string(),
                                    "recipes" => "Recipe Costing".to_string(),
                                    "vendors" => "Vendor Hub".to_string(),
                                    "conversions" => "Conversion Lab".to_string(),
                                    "reports" => "Data Health".to_string(),
                                    "purchasing" => "Purchasing".to_string(),
                                    "foodcost" => "Food Cost Calculator".to_string(),
                                    "settings" => "General Settings".to_string(),
                                    "fda" => "FDA Guidelines".to_string(),
                                    _ => "4chef".to_string(),
                                }
                            }}
                        </div>
                        <div class="subtitle">
                            {move || {
                                match active_panel.get().as_str() {
                                    "inventory" => "Search and inspect inventory items in your 4chef library."
                                        .to_string(),
                                    "recipes" => "Cost recipes using purchase units and conversions."
                                        .to_string(),
                                    "vendors" => "Search vendors and review their priced items.".to_string(),
                                    "conversions" => "Review conversion gaps and suggested fixes.".to_string(),
                                    "reports" => "Track missing data that blocks accurate costing.".to_string(),
                                    "purchasing" => "Browse invoices and transaction lines.".to_string(),
                                    "foodcost" => "Build a dish, set your target %, and find the right menu price.".to_string(),
                                    "settings" => "Company info and preferences.".to_string(),
                                    "fda" => "Quick-reference FDA food safety guidelines for your kitchen.".to_string(),
                                    _ => "Organize. Optimize. Itemize. Done."
                                        .to_string(),
                                }
                            }}
                        </div>
                    </div>
                </div>

                <Show when=move || active_panel.get() == "imports">
                    <div class="panel">
                        <div class="row">
                            <div class="input" style="flex: 1;">
                                <label>
                                    {move || {
                                        if import_mode.get() == "mdf" {
                                            "MDF/LDF source"
                                        } else {
                                            "Exports folder"
                                        }
                                    }}
                                </label>
                                <div style="display: flex; gap: 6px;">
                                    <input
                                        type="text"
                                        prop:value=base_path
                                        on:input=move |ev| {
                                            let value = event_target_value(&ev);
                                            set_base_path.set(value);
                                        }
                                        style="flex: 1;"
                                    />
                                    <button
                                        class="button secondary"
                                        style="white-space: nowrap;"
                                        on:click=move |_| {
                                            let mode_now = import_mode.get();
                                            spawn_local(async move {
                                                if let Some(w) = window() {
                                                    if let Ok(tauri) = Reflect::get(&w, &JsValue::from_str("__TAURI__")) {
                                                        if let Ok(dialog) = Reflect::get(&tauri, &JsValue::from_str("dialog")) {
                                                            if let Ok(open_fn) = Reflect::get(&dialog, &JsValue::from_str("open")) {
                                                                let opts = js_sys::Object::new();
                                                                let _ = Reflect::set(
                                                                    &opts,
                                                                    &JsValue::from_str("directory"),
                                                                    &JsValue::from_bool(mode_now != "mdf"),
                                                                );
                                                                let _ = Reflect::set(&opts, &JsValue::from_str("multiple"), &JsValue::from_bool(false));
                                                                let title = if mode_now == "mdf" {
                                                                    "Select MDF or LDF file"
                                                                } else {
                                                                    "Select exports folder"
                                                                };
                                                                let _ = Reflect::set(
                                                                    &opts,
                                                                    &JsValue::from_str("title"),
                                                                    &JsValue::from_str(title),
                                                                );
                                                                if let Ok(f) = open_fn.dyn_ref::<js_sys::Function>()
                                                                    .ok_or(JsValue::NULL)
                                                                {
                                                                    if let Ok(promise) = f.call1(&JsValue::NULL, &opts) {
                                                                        if let Ok(p) = promise.dyn_into::<Promise>() {
                                                                            if let Ok(result) = JsFuture::from(p).await {
                                                                                if let Some(path) = result.as_string() {
                                                                                    set_base_path.set(path);
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    >
                                        "Browse\u{2026}"
                                    </button>
                                </div>
                            </div>
                            <div class="input" style="max-width: 220px;">
                                <label>"Import mode"</label>
                                <select
                                    prop:value=move || import_mode.get()
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_import_mode.set(value);
                                    }
                                >
                                    <option value="exports">"Exports (CSV)"</option>
                                    <option value="mdf">"MDF/LDF database"</option>
                                </select>
                            </div>
                            <div class="input">
                                <label style="visibility: hidden;">{"\u{00A0}"}</label>
                                <button class="button import-btn" on:click=move |_| start_import() disabled=move || busy.get()>
                                    {move || if busy.get() { "Importing\u{2026}" } else { "\u{2B07} Import" }}
                                </button>
                            </div>
                        </div>
                    <div class="status">{move || status.get()}</div>
                    <div class="status">{move || tauri_status.get()}</div>
                    <div class="row" style="margin-top: 10px;">
                        <button class="button secondary" on:click=move |_| ping_backend()>
                            "Ping backend"
                        </button>
                        <button class="button secondary" on:click=move |_| open_path(base_path.get())>
                            {move || {
                                if import_mode.get() == "mdf" {
                                    "Open source path"
                                } else {
                                    "Open exports folder"
                                }
                            }}
                        </button>
                        <div class="status">{move || ping_status.get()}</div>
                    </div>
                </div>

                    <div class="panel">
                        <div class="row">
                            <div>
                                <strong>"Database"</strong>
                                <div class="status">
                                    {move || if has_summary.get() { summary.get().db_path } else { "No database yet".to_string() }}
                                </div>
                            </div>
                            <div style="margin-left: auto;">
                                <button class="button secondary" on:click=move |_| {
                                    let path = summary.get().db_path;
                                    open_path(path);
                                } disabled=move || !has_summary.get()>
                                    "Open DB folder"
                                </button>
                            </div>
                        </div>
                        <div class="cards">
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "units"
                                on:click=move |_| toggle_card("units")
                            >
                                <h4>"Units"</h4>
                                <p>{move || summary.get().units.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "items"
                                on:click=move |_| toggle_card("items")
                            >
                                <h4>"Items"</h4>
                                <p>{move || summary.get().items.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "convunit"
                                on:click=move |_| toggle_card("convunit")
                            >
                                <h4>"Conversions"</h4>
                                <p>{move || summary.get().convunit.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "recp_items"
                                on:click=move |_| toggle_card("recp_items")
                            >
                                <h4>"Recipe Items"</h4>
                                <p>{move || summary.get().recp_items.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "inv_units"
                                on:click=move |_| toggle_card("inv_units")
                            >
                                <h4>"Inv Units"</h4>
                                <p>{move || summary.get().inv_units.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "vendors"
                                on:click=move |_| toggle_card("vendors")
                            >
                                <h4>"Vendors"</h4>
                                <p>{move || summary.get().vendors.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "recipes"
                                on:click=move |_| toggle_card("recipes")
                            >
                                <h4>"Recipes"</h4>
                                <p>{move || summary.get().recipes.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "inv_prices"
                                on:click=move |_| toggle_card("inv_prices")
                            >
                                <h4>"Inv Prices"</h4>
                                <p>{move || summary.get().inv_prices.to_string()}</p>
                            </div>
                        </div>
                        <div class="cards">
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "invoices"
                                on:click=move |_| toggle_card("invoices")
                            >
                                <h4>"Invoices"</h4>
                                <p>{move || summary.get().invoices.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "trans"
                                on:click=move |_| toggle_card("trans")
                            >
                                <h4>"Trans Lines"</h4>
                                <p>{move || summary.get().trans.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "recp_inv"
                                on:click=move |_| toggle_card("recp_inv")
                            >
                                <h4>"RecpInv"</h4>
                                <p>{move || summary.get().recp_inv.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "conv_suggestions"
                                on:click=move |_| toggle_card("conv_suggestions")
                            >
                                <h4>"Conv Suggestions"</h4>
                                <p>{move || summary.get().conv_suggestions.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "conv_suggestions_safe"
                                on:click=move |_| toggle_card("conv_suggestions_safe")
                            >
                                <h4>"Safe Suggestions"</h4>
                                <p>{move || summary.get().conv_suggestions_safe.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "missing_edges"
                                on:click=move |_| toggle_card("missing_edges")
                            >
                                <h4>"Local Conversions"</h4>
                                <p>{move || summary.get().missing_edges.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "missing_data_report"
                                on:click=move |_| toggle_card("missing_data_report")
                            >
                                <h4>"Missing Data"</h4>
                                <p>{move || summary.get().missing_data_report.to_string()}</p>
                            </div>
                            <div
                                class="card card-clickable"
                                class:card-active=move || browse_table_name.get() == "missing_purch_unit"
                                on:click=move |_| toggle_card("missing_purch_unit")
                            >
                                <h4>"Missing Purch"</h4>
                                <p>{move || summary.get().missing_purch_unit.to_string()}</p>
                            </div>
                        </div>

                        <Show when=move || !browse_table_name.get().is_empty()>
                            <div class="browse-section">
                                <div class="row" style="align-items: center; margin-bottom: 8px;">
                                    <strong>{move || format!("Browsing: {}", browse_table_name.get())}</strong>
                                    <span class="status" style="margin-left: 10px;">
                                        {move || {
                                            let data = browse_data.get();
                                            let page = browse_page.get();
                                            let start = page * browse_limit + 1;
                                            let end = std::cmp::min((page + 1) * browse_limit, data.total as u32);
                                            if data.total > 0 {
                                                format!("Rows {}\u{2013}{} of {}", start, end, data.total)
                                            } else {
                                                "No rows".to_string()
                                            }
                                        }}
                                    </span>
                                    <div style="margin-left: auto; display: flex; gap: 6px;">
                                        <button
                                            class="button secondary"
                                            disabled=move || browse_page.get() == 0 || browse_loading.get()
                                            on:click=move |_| {
                                                let p = browse_page.get().saturating_sub(1);
                                                set_browse_page.set(p);
                                                fetch_browse_table(browse_table_name.get(), p);
                                            }
                                        >
                                            "\u{25C0} Prev"
                                        </button>
                                        <button
                                            class="button secondary"
                                            disabled=move || {
                                                let data = browse_data.get();
                                                ((browse_page.get() + 1) * browse_limit) as i64 >= data.total || browse_loading.get()
                                            }
                                            on:click=move |_| {
                                                let p = browse_page.get() + 1;
                                                set_browse_page.set(p);
                                                fetch_browse_table(browse_table_name.get(), p);
                                            }
                                        >
                                            "Next \u{25B6}"
                                        </button>
                                    </div>
                                </div>
                                <Show when=move || browse_loading.get()>
                                    <div class="status">"Loading\u{2026}"</div>
                                </Show>
                                <Show when=move || !browse_loading.get() && !browse_data.get().columns.is_empty()>
                                    <div class="browse-table-wrap">
                                        <table class="browse-table">
                                            <thead>
                                                <tr>
                                                    <For
                                                        each=move || browse_data.get().columns.clone()
                                                        key=|col| col.clone()
                                                        children=move |col| {
                                                            let label = humanize_column(&col);
                                                            view! { <th>{label}</th> }
                                                        }
                                                    />
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || {
                                                        browse_data.get().rows.iter().enumerate()
                                                            .map(|(i, r)| (i, r.clone()))
                                                            .collect::<Vec<_>>()
                                                    }
                                                    key=|(i, _)| *i
                                                    children=move |(_, row)| view! {
                                                        <tr>
                                                            <For
                                                                each=move || {
                                                        row.iter().enumerate()
                                                            .map(|(i, c)| (i, c.clone()))
                                                            .collect::<Vec<_>>()
                                                    }
                                                    key=|(i, _)| *i
                                                    children=move |(_, cell)| view! { <td>{cell}</td> }
                                                            />
                                                        </tr>
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                </Show>
                            </div>
                        </Show>
                    </div>

                    <Show
                        when=move || has_summary.get() && !summary.get().warnings.is_empty()
                    >
                        <div class="panel">
                            <div class="row">
                                <div>
                                    <strong>"Import Notes"</strong>
                                    <div class="status">
                                        {move || format!("Warnings: {}", summary.get().warnings.len())}
                                    </div>
                                </div>
                                <div style="margin-left: auto;">
                                    <button class="button secondary" on:click=move |_| {
                                        set_show_warnings.set(!show_warnings.get());
                                    }>
                                        {move || if show_warnings.get() { "Hide details" } else { "View details" }}
                                    </button>
                                </div>
                            </div>
                            <Show when=move || show_warnings.get()>
                                <div class="log">
                                    <For
                                        each=move || summary.get().warnings
                                        key=|msg| msg.clone()
                                        children=move |msg| view! { <div>{msg}</div> }
                                    />
                                </div>
                            </Show>
                        </div>
                    </Show>

                    // ── PDF Invoice Import ──
                    <div class="panel">
                        <div class="row">
                            <div>
                                <strong>"Import PDF Invoice"</strong>
                                <div class="status">"Upload a digital vendor invoice (PDF) to preview and import line items."</div>
                            </div>
                        </div>
                        <div class="row" style="margin-top: 12px;">
                            <button
                                class="button secondary"
                                on:click=move |_| {
                                    spawn_local(async move {
                                        let w = match window() {
                                            Some(w) => w,
                                            None => return,
                                        };
                                        let tauri = match Reflect::get(&w, &JsValue::from_str("__TAURI__")) {
                                            Ok(t) if !t.is_undefined() => t,
                                            _ => return,
                                        };
                                        let dialog = match Reflect::get(&tauri, &JsValue::from_str("dialog")) {
                                            Ok(d) if !d.is_undefined() => d,
                                            _ => return,
                                        };
                                        let open_fn = match Reflect::get(&dialog, &JsValue::from_str("open")) {
                                            Ok(f) if f.is_function() => f,
                                            _ => return,
                                        };
                                        let opts = js_sys::Object::new();
                                        let _ = Reflect::set(&opts, &JsValue::from_str("title"), &JsValue::from_str("Select PDF Invoice"));
                                        let _ = Reflect::set(&opts, &JsValue::from_str("directory"), &JsValue::from_bool(false));
                                        let _ = Reflect::set(&opts, &JsValue::from_str("multiple"), &JsValue::from_bool(false));
                                        let filter = js_sys::Object::new();
                                        let _ = Reflect::set(&filter, &JsValue::from_str("name"), &JsValue::from_str("PDF"));
                                        let exts = js_sys::Array::new();
                                        exts.push(&JsValue::from_str("pdf"));
                                        let _ = Reflect::set(&filter, &JsValue::from_str("extensions"), &exts);
                                        let filters = js_sys::Array::new();
                                        filters.push(&filter);
                                        let _ = Reflect::set(&opts, &JsValue::from_str("filters"), &filters);

                                        let func = open_fn.dyn_ref::<js_sys::Function>().unwrap();
                                        if let Ok(promise) = func.call1(&JsValue::NULL, &opts) {
                                            if let Ok(p) = promise.dyn_into::<Promise>() {
                                                if let Ok(result) = JsFuture::from(p).await {
                                                    if let Some(path) = result.as_string() {
                                                        set_pdf_import_status.set("Extracting PDF...".to_string());
                                                        let args = to_value(&PdfPathArgs { pdf_path: path }).unwrap();
                                                        match invoke_cmd::<PdfInvoicePreview>("preview_pdf_invoice", args).await {
                                                            Ok(preview) => {
                                                                set_pdf_invoice_no.set(preview.invoice_no.clone());
                                                                set_pdf_invoice_date.set(preview.invoice_date.clone());
                                                                set_pdf_import_status.set(format!(
                                                                    "Extracted {} line items from PDF",
                                                                    preview.lines.len()
                                                                ));
                                                                set_pdf_preview.set(Some(preview));
                                                            }
                                                            Err(err) => {
                                                                set_pdf_import_status.set(format!("PDF extraction failed: {err}"));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                                }
                            >
                                "Select PDF File"
                            </button>
                        </div>
                        <div class="status">{move || pdf_import_status.get()}</div>

                        <Show when=move || pdf_preview.get().is_some()>
                            {move || pdf_preview.with(|opt| {
                                opt.as_ref().map(|preview| {
                                    let preview = preview.clone();
                                    let line_count = preview.lines.len();
                                    view! {
                                        <div style="margin-top: 12px;">
                                            <div class="row">
                                                <div class="input">
                                                    <label>"Vendor"</label>
                                                    <select
                                                        prop:value=pdf_vendor_id
                                                        on:change=move |ev| {
                                                            set_pdf_vendor_id.set(event_target_value(&ev));
                                                        }
                                                    >
                                                        <option value="">{format!("Detected: {}", preview.vendor_guess)}</option>
                                                        <For
                                                            each=move || editor_vendor_options.get()
                                                            key=|v| v.vendor_id
                                                            children=move |v| view! {
                                                                <option value={v.vendor_id.to_string()}>{format!("{} - {}", v.vendor_id, v.name)}</option>
                                                            }
                                                        />
                                                    </select>
                                                </div>
                                                <div class="input">
                                                    <label>"Invoice #"</label>
                                                    <input
                                                        type="text"
                                                        prop:value=pdf_invoice_no
                                                        on:input=move |ev| {
                                                            set_pdf_invoice_no.set(event_target_value(&ev));
                                                        }
                                                    />
                                                </div>
                                                <div class="input">
                                                    <label>"Date"</label>
                                                    <input
                                                        type="text"
                                                        prop:value=pdf_invoice_date
                                                        on:input=move |ev| {
                                                            set_pdf_invoice_date.set(event_target_value(&ev));
                                                        }
                                                    />
                                                </div>
                                            </div>
                                            <div class="data-table" style="margin-top: 12px;">
                                                <div class="data-header data-cols-5">
                                                    <span>"Item"</span>
                                                    <span>"Qty"</span>
                                                    <span>"Unit"</span>
                                                    <span>"Unit Price"</span>
                                                    <span>"Ext. Cost"</span>
                                                </div>
                                                <For
                                                    each=move || {
                                                        let v: Vec<(usize, PdfInvoiceLine)> = preview.lines.clone().into_iter().enumerate().collect();
                                                        v
                                                    }
                                                    key=|(i, _)| *i
                                                    children=move |(_, line)| view! {
                                                        <div class="data-row data-cols-5">
                                                            <span>{line.item_name}</span>
                                                            <span>{line.qty.map(|q| format!("{:.2}", q)).unwrap_or_else(|| "-".to_string())}</span>
                                                            <span>{line.unit}</span>
                                                            <span>{line.unit_price.map(|p| format!("${:.2}", p)).unwrap_or_else(|| "-".to_string())}</span>
                                                            <span>{line.ext_cost.map(|c| format!("${:.2}", c)).unwrap_or_else(|| "-".to_string())}</span>
                                                        </div>
                                                    }
                                                />
                                            </div>
                                            <div class="row" style="margin-top: 12px;">
                                                <button
                                                    class="button import-btn"
                                                    on:click=move |_| {
                                                        let vid = match pdf_vendor_id.get().parse::<i64>() {
                                                            Ok(v) => v,
                                                            Err(_) => {
                                                                set_pdf_import_status.set("Please select a vendor".to_string());
                                                                return;
                                                            }
                                                        };
                                                        let inv_no = pdf_invoice_no.get();
                                                        let inv_date = pdf_invoice_date.get();
                                                        let pdf_lines = pdf_preview.get().map(|p| p.lines.clone()).unwrap_or_default();
                                                        set_pdf_import_status.set("Importing...".to_string());
                                                        spawn_local(async move {
                                                            let line_vals: Vec<serde_json::Value> = pdf_lines.iter().map(|l| {
                                                                serde_json::json!({
                                                                    "item_name": l.item_name,
                                                                    "qty": l.qty,
                                                                    "unit_price": l.unit_price,
                                                                    "ext_cost": l.ext_cost,
                                                                })
                                                            }).collect();
                                                            let args = to_value(&ImportPdfInvoiceArgs {
                                                                vendor_id: vid,
                                                                invoice_no: inv_no,
                                                                invoice_date: inv_date,
                                                                lines: line_vals,
                                                            }).unwrap();
                                                            match invoke_cmd::<PatchResponse>("import_pdf_invoice", args).await {
                                                                Ok(resp) => {
                                                                    set_pdf_import_status.set(resp.message);
                                                                    set_pdf_preview.set(None);
                                                                }
                                                                Err(err) => set_pdf_import_status.set(format!("Import failed: {err}")),
                                                            }
                                                        });
                                                    }
                                                >
                                                    {move || format!("Import {} Lines", line_count)}
                                                </button>
                                                <button
                                                    class="button secondary"
                                                    on:click=move |_| {
                                                        set_pdf_preview.set(None);
                                                        set_pdf_import_status.set(String::new());
                                                    }
                                                >
                                                    "Cancel"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                })
                            })}
                        </Show>
                    </div>
                </Show>

                <Show when=move || active_panel.get() == "inventory">
                    <div class="panel">
                        <div class="row">
                            <div class="input">
                                <label>"Search"</label>
                                <input
                                    type="text"
                                    prop:value=inventory_query
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_inventory_query.set(value);
                                    }
                                />
                            </div>
                            <div class="input" style="align-self: end;">
                                <div class="row">
                                    <button class="button" on:click=move |_| {
                                        set_inventory_page.set(0);
                                        trigger_inventory_fetch(
                                            inventory_query.get(),
                                            0,
                                            inventory_limit,
                                            set_inventory_loading,
                                            set_inventory_status,
                                            set_inventory_items,
                                            set_inventory_total,
                                            set_inventory_filtered,
                                            set_inventory_loaded,
                                        );
                                    } disabled=move || inventory_loading.get()>
                                        {move || if inventory_loading.get() { "Searching..." } else { "Search" }}
                                    </button>
                                    <button class="button secondary" on:click=move |_| {
                                        set_inventory_query.set(String::new());
                                        set_inventory_page.set(0);
                                        trigger_inventory_fetch(
                                            String::new(),
                                            0,
                                            inventory_limit,
                                            set_inventory_loading,
                                            set_inventory_status,
                                            set_inventory_items,
                                            set_inventory_total,
                                            set_inventory_filtered,
                                            set_inventory_loaded,
                                        );
                                    }>
                                        "Reset"
                                    </button>
                                </div>
                            </div>
                        </div>
                        <div class="status">{move || inventory_status.get()}</div>
                    </div>

                    <div class="panel inventory-grid">
                        <div class="inventory-list">
                            <div class="row">
                                <div>
                                    <strong>"Inventory Overview"</strong>
                                    <div class="status">
                                        {move || format!(
                                            "Total items: {} | Filtered: {} | Page {}",
                                            inventory_total.get(),
                                            inventory_filtered.get(),
                                            inventory_page.get() + 1
                                        )}
                                    </div>
                                </div>
                                <div style="margin-left: auto; display: flex; gap: 8px;">
                                    <button class="button tiny" on:click=move |_| export_inventory_pdf()>"Export PDF"</button>
                                    <button class="button tiny secondary" on:click=move |_| export_inventory_docx()>"Export Word"</button>
                                </div>
                            </div>
                            <Show when=move || !export_status.get().is_empty()>
                                <div class="status">{move || export_status.get()}</div>
                            </Show>
                            <div class="inventory-table">
                                <div class="inventory-header">
                                    <span>"Item ID"</span>
                                    <span>"Name"</span>
                                    <span>"Status"</span>
                                </div>
                                <For
                                    each=move || inventory_items.get()
                                    key=|item| item.item_id
                                    children=move |item| {
                                        let item_id = item.item_id;
                                        view! {
                                            <div
                                                class="inventory-row"
                                                class:selected=move || inventory_selected.get().map(|d| d.item_id == item_id).unwrap_or(false)
                                                on:click=move |_| {
                                                    trigger_inventory_detail_fetch(
                                                        item_id,
                                                        set_inventory_detail_loading,
                                                        set_inventory_detail_status,
                                                        set_inventory_selected,
                                                    );
                                                }
                                            >
                                                <span>{item.item_id}</span>
                                                <span>{item.name}</span>
                                                <span>{item.status.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            <div class="row" style="margin-top: 16px;">
                                <button class="button secondary" on:click=move |_| {
                                    let current = inventory_page.get();
                                    if current > 0 {
                                        set_inventory_page.set(current - 1);
                                        trigger_inventory_fetch(
                                            inventory_query.get(),
                                            current - 1,
                                            inventory_limit,
                                            set_inventory_loading,
                                            set_inventory_status,
                                            set_inventory_items,
                                            set_inventory_total,
                                            set_inventory_filtered,
                                            set_inventory_loaded,
                                        );
                                    }
                                } disabled=move || inventory_page.get() == 0 || inventory_loading.get()>
                                    "Previous"
                                </button>
                                <button class="button" on:click=move |_| {
                                    let current = inventory_page.get();
                                    let max = (inventory_filtered.get() as f64 / inventory_limit as f64).ceil() as usize;
                                    if current + 1 < max {
                                        set_inventory_page.set(current + 1);
                                        trigger_inventory_fetch(
                                            inventory_query.get(),
                                            current + 1,
                                            inventory_limit,
                                            set_inventory_loading,
                                            set_inventory_status,
                                            set_inventory_items,
                                            set_inventory_total,
                                            set_inventory_filtered,
                                            set_inventory_loaded,
                                        );
                                    }
                                } disabled=move || {
                                    let max = (inventory_filtered.get() as f64 / inventory_limit as f64).ceil() as usize;
                                    inventory_loading.get() || max == 0 || inventory_page.get() + 1 >= max
                                }>
                                    "Next"
                                </button>
                            </div>
                        </div>
                        <div class="inventory-detail">
                            <div class="status">{move || inventory_detail_status.get()}</div>
                            <Show
                                when=move || inventory_selected.get().is_some()
                                fallback=move || view! {
                                    <div class="recipe-empty">
                                        <strong>"Select an item"</strong>
                                        <p>"Choose an inventory item to review purchase units, pricing, and usage."</p>
                                    </div>
                                }
                            >
                                {move || inventory_selected.with(|opt| {
                                    opt.as_ref().map(|detail| {
                                        let detail = detail.clone();
                                        view! {
                                            <div>
                                                <div class="recipe-title">{detail.name.clone()}</div>
                                                <div class="recipe-summary">
                                                    <span>{format!("Item ID {}", detail.item_id)}</span>
                                                    <span>{format!("Status {}", detail.status.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()))}</span>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Edit Item"</strong>
                                                    <div class="row" style="margin-top: 10px;">
                                                        <div class="input">
                                                            <label>"Name"</label>
                                                            <input
                                                                type="text"
                                                                prop:value=move || {
                                                                    let v = edit_item_name.get();
                                                                    if v.is_empty() {
                                                                        inventory_selected.get()
                                                                            .map(|d| d.name.clone())
                                                                            .unwrap_or_default()
                                                                    } else { v }
                                                                }
                                                                on:input=move |ev| {
                                                                    set_edit_item_name.set(event_target_value(&ev));
                                                                }
                                                                on:focus=move |_| {
                                                                    if edit_item_name.get().is_empty() {
                                                                        if let Some(d) = inventory_selected.get() {
                                                                            set_edit_item_name.set(d.name.clone());
                                                                        }
                                                                    }
                                                                }
                                                            />
                                                        </div>
                                                        <div class="input" style="flex: 0 0 100px;">
                                                            <label>"Status"</label>
                                                            <input
                                                                type="number"
                                                                prop:value=move || {
                                                                    let v = edit_item_status.get();
                                                                    if v.is_empty() {
                                                                        inventory_selected.get()
                                                                            .and_then(|d| d.status)
                                                                            .map(|s| s.to_string())
                                                                            .unwrap_or_default()
                                                                    } else { v }
                                                                }
                                                                on:input=move |ev| {
                                                                    set_edit_item_status.set(event_target_value(&ev));
                                                                }
                                                            />
                                                        </div>
                                                        <div class="input" style="align-self: end; flex: 0 0 auto;">
                                                            <button
                                                                class="button tiny"
                                                                on:click=move |_| {
                                                                    save_item_edit(detail.item_id);
                                                                    set_edit_item_name.set(String::new());
                                                                    set_edit_item_status.set(String::new());
                                                                }
                                                            >
                                                                "Save changes"
                                                            </button>
                                                        </div>
                                                    </div>
                                                    <div class="status">{move || edit_item_msg.get()}</div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Assign Purchase Unit"</strong>
                                                    <div class="row" style="margin-top: 10px;">
                                                        <div class="input">
                                                            <label>"Unit"</label>
                                                            <select
                                                                prop:value=purch_unit_selected_id
                                                                on:change=move |ev| {
                                                                    set_purch_unit_selected_id.set(event_target_value(&ev));
                                                                }
                                                            >
                                                                <option value="">"Select unit"</option>
                                                                <For
                                                                    each=move || unit_options.get()
                                                                    key=|unit| unit.unit_id
                                                                    children=move |unit| view! {
                                                                        <option value={unit.unit_id.to_string()}>{format!("{} - {}", unit.unit_id, unit.sing)}</option>
                                                                    }
                                                                />
                                                            </select>
                                                        </div>
                                                        <div class="input" style="flex: 0 0 150px;">
                                                            <label>"Default"</label>
                                                            <label style="display: inline-flex; gap: 8px; align-items: center;">
                                                                <input
                                                                    type="checkbox"
                                                                    prop:checked=purch_unit_set_default
                                                                    on:change=move |ev| {
                                                                        set_purch_unit_set_default.set(event_target_checked(&ev));
                                                                    }
                                                                />
                                                                <span>"Set default"</span>
                                                            </label>
                                                        </div>
                                                        <div class="input" style="align-self: end; flex: 0 0 auto;">
                                                            <button
                                                                class="button tiny"
                                                                on:click=move |_| assign_purch_unit(detail.item_id)
                                                            >
                                                                "Save unit"
                                                            </button>
                                                        </div>
                                                    </div>
                                                    <div class="status">{move || purch_unit_status.get()}</div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Purchase Units"</strong>
                                                    <div class="data-table">
                                                        <div class="data-header data-cols-3">
                                                            <span>"Unit"</span>
                                                            <span>"Default"</span>
                                                            <span>"Status"</span>
                                                        </div>
                                                        <For
                                                            each=move || detail.purch_units.clone()
                                                            key=|unit| unit.unit_id.unwrap_or(0)
                                                            children=move |unit| view! {
                                                                <div class="data-row data-cols-3">
                                                                    <span>{unit.unit_name}</span>
                                                                    <span>{unit.is_default.map(|v| if v == 1 { "Yes" } else { "No" }.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                                                    <span>{unit.status.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                                                </div>
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Vendor Prices"</strong>
                                                    <div class="data-table">
                                                        <div class="data-header data-cols-6">
                                                            <span>"Vendor"</span>
                                                            <span>"Current Price"</span>
                                                            <span>"Previous Price"</span>
                                                            <span>"Diff %"</span>
                                                            <span>"Pack"</span>
                                                            <span>"Status"</span>
                                                        </div>
                                                        <For
                                                            each=move || detail.prices.clone()
                                                            key=|price| price.vendor_id.unwrap_or(0)
                                                            children=move |price| {
                                                                let diff_class = match price.diff_pct {
                                                                    Some(d) if d > 0.0 => "diff-up",
                                                                    Some(d) if d < 0.0 => "diff-down",
                                                                    Some(_) => "diff-zero",
                                                                    None => "",
                                                                };
                                                                let diff_text = match price.diff_pct {
                                                                    Some(d) => format!("{:+.1}%", d),
                                                                    None => "-".to_string(),
                                                                };
                                                                view! {
                                                                    <div class="data-row data-cols-6">
                                                                        <span>{price.vendor_name}</span>
                                                                        <span>{price.price.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                                        <span>{price.prev_price.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                                        <span class={diff_class}>{diff_text}</span>
                                                                        <span>{price.pack}</span>
                                                                        <span>{price.status.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Manual Price Override"</strong>
                                                    <div class="row" style="margin-top: 10px;">
                                                        <div class="input">
                                                            <label>"Vendor"</label>
                                                            <select
                                                                prop:value=manual_price_vendor_id
                                                                on:change=move |ev| {
                                                                    set_manual_price_vendor_id.set(event_target_value(&ev));
                                                                }
                                                            >
                                                                <option value="">"Select vendor"</option>
                                                                <For
                                                                    each=move || editor_vendor_options.get()
                                                                    key=|vendor| vendor.vendor_id
                                                                    children=move |vendor| view! {
                                                                        <option value={vendor.vendor_id.to_string()}>{format!("{} - {}", vendor.vendor_id, vendor.name)}</option>
                                                                    }
                                                                />
                                                            </select>
                                                        </div>
                                                        <div class="input">
                                                            <label>"Price"</label>
                                                            <input
                                                                type="number"
                                                                step="0.0001"
                                                                min="0"
                                                                prop:value=manual_price_value
                                                                on:input=move |ev| {
                                                                    set_manual_price_value.set(event_target_value(&ev));
                                                                }
                                                            />
                                                        </div>
                                                        <div class="input">
                                                            <label>"Pack"</label>
                                                            <input
                                                                type="text"
                                                                prop:value=manual_price_pack
                                                                on:input=move |ev| {
                                                                    set_manual_price_pack.set(event_target_value(&ev));
                                                                }
                                                            />
                                                        </div>
                                                        <div class="input" style="align-self: end; flex: 0 0 auto;">
                                                            <button
                                                                class="button tiny"
                                                                on:click=move |_| save_manual_price(detail.item_id)
                                                            >
                                                                "Save price"
                                                            </button>
                                                        </div>
                                                    </div>
                                                    <div class="status">{move || manual_price_status.get()}</div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Conversions"</strong>
                                                    <div class="data-table">
                                                        <div class="data-header data-cols-5">
                                                            <span>"Vendor"</span>
                                                            <span>"Unit 1"</span>
                                                            <span>"Qty 1"</span>
                                                            <span>"Unit 2"</span>
                                                            <span>"Qty 2"</span>
                                                        </div>
                                                        <For
                                                            each=move || detail.conversions.clone()
                                                            key=|conv| (conv.unit_id1, conv.unit_id2, conv.vendor_id)
                                                            children=move |conv| view! {
                                                                <div class="data-row data-cols-5">
                                                                    <span>{conv.vendor_id}</span>
                                                                    <span>{conv.unit_id1}</span>
                                                                    <span>{format!("{:.3}", conv.qty1)}</span>
                                                                    <span>{conv.unit_id2}</span>
                                                                    <span>{format!("{:.3}", conv.qty2)}</span>
                                                                </div>
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Recipe Usage"</strong>
                                                    <div class="data-table">
                                                        <div class="data-header data-cols-3">
                                                            <span>"Recipe"</span>
                                                            <span>"Qty"</span>
                                                            <span>"Unit"</span>
                                                        </div>
                                                        <For
                                                            each=move || detail.usage.clone()
                                                            key=|usage| usage.recipe_id
                                                            children=move |usage| view! {
                                                                <div class="data-row data-cols-3">
                                                                    <span>{usage.recipe_name}</span>
                                                                    <span>{usage.qty.map(|q| format!("{:.3}", q)).unwrap_or_else(|| "-".to_string())}</span>
                                                                    <span>{usage.unit_name}</span>
                                                                </div>
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Local Conversions"</strong>
                                                    <div class="data-table">
                                                        <div class="data-header data-cols-4">
                                                            <span>"Vendor"</span>
                                                            <span>"Recipe Unit"</span>
                                                            <span>"Purch Unit"</span>
                                                            <span>"Hits"</span>
                                                        </div>
                                                        <For
                                                            each=move || detail.missing_edges.clone()
                                                            key=|edge| (edge.vendor_id, edge.recipe_unit.clone(), edge.purch_unit.clone())
                                                            children=move |edge| view! {
                                                                <div class="data-row data-cols-4">
                                                                    <span>{edge.vendor_id}</span>
                                                                    <span>{edge.recipe_unit}</span>
                                                                    <span>{edge.purch_unit}</span>
                                                                    <span>{edge.hits.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                                                </div>
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })
                                })}
                            </Show>
                            <Show when=move || inventory_detail_loading.get()>
                                <div class="status">"Loading item detail..."</div>
                            </Show>
                        </div>
                    </div>
                </Show>

                <Show when=move || active_panel.get() == "recipes">
                    <div class="panel">
                        <div class="row">
                            <div class="input">
                                <label>"Search recipes"</label>
                                <input
                                    type="text"
                                    prop:value=recipe_query
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_recipe_query.set(value);
                                    }
                                />
                            </div>
                            <div class="input" style="align-self: end;">
                                <div class="row">
                                    <button class="button" on:click=move |_| {
                                        set_recipe_page.set(0);
                                        trigger_recipe_fetch(
                                            recipe_query.get(),
                                            0,
                                            recipe_limit,
                                            set_recipe_loading,
                                            set_recipe_status,
                                            set_recipe_list,
                                            set_recipe_total,
                                            set_recipe_filtered,
                                            set_recipe_loaded,
                                            true,
                                            set_recipe_detail_loading,
                                            set_recipe_detail_status,
                                            set_selected_recipe,
                                        );
                                    } disabled=move || recipe_loading.get()>
                                        {move || if recipe_loading.get() { "Searching..." } else { "Search" }}
                                    </button>
                                    <button class="button secondary" on:click=move |_| {
                                        set_recipe_query.set(String::new());
                                        set_recipe_page.set(0);
                                        trigger_recipe_fetch(
                                            String::new(),
                                            0,
                                            recipe_limit,
                                            set_recipe_loading,
                                            set_recipe_status,
                                            set_recipe_list,
                                            set_recipe_total,
                                            set_recipe_filtered,
                                            set_recipe_loaded,
                                            true,
                                            set_recipe_detail_loading,
                                            set_recipe_detail_status,
                                            set_selected_recipe,
                                        );
                                    }>
                                        "Reset"
                                    </button>
                                </div>
                            </div>
                        </div>
                        <div class="status">{move || recipe_status.get()}</div>
                    </div>

                    <div class="panel recipe-grid">
                        <div class="recipe-list">
                            <div class="recipe-list-header">
                                <span>"Recipes"</span>
                                <span>{move || format!(
                                    "{} / {}",
                                    recipe_filtered.get(),
                                    recipe_total.get()
                                )}</span>
                            </div>
                            <div class="recipe-list-body">
                                <For
                                    each=move || recipe_list.get()
                                    key=|item| item.recipe_id
                                    children=move |item| {
                                        let recipe_id = item.recipe_id;
                                        view! {
                                            <div
                                                class="recipe-row"
                                                class:selected=move || selected_recipe.get().map(|r| r.recipe_id == recipe_id).unwrap_or(false)
                                                on:click=move |_| {
                                                    trigger_recipe_detail_fetch(
                                                        recipe_id,
                                                        set_recipe_detail_loading,
                                                        set_recipe_detail_status,
                                                        set_selected_recipe,
                                                    );
                                                }
                                            >
                                                <div class="recipe-name">{item.name}</div>
                                                <div class="recipe-meta">{format!("{} items", item.item_count)}</div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            <div class="row" style="margin-top: 12px;">
                                <button class="button secondary" on:click=move |_| {
                                    let current = recipe_page.get();
                                    if current > 0 {
                                        set_recipe_page.set(current - 1);
                                        trigger_recipe_fetch(
                                            recipe_query.get(),
                                            current - 1,
                                            recipe_limit,
                                            set_recipe_loading,
                                            set_recipe_status,
                                            set_recipe_list,
                                            set_recipe_total,
                                            set_recipe_filtered,
                                            set_recipe_loaded,
                                            true,
                                            set_recipe_detail_loading,
                                            set_recipe_detail_status,
                                            set_selected_recipe,
                                        );
                                    }
                                } disabled=move || recipe_page.get() == 0 || recipe_loading.get()>
                                    "Previous"
                                </button>
                                <button class="button" on:click=move |_| {
                                    let current = recipe_page.get();
                                    let max = (recipe_filtered.get() as f64 / recipe_limit as f64).ceil() as usize;
                                    if current + 1 < max {
                                        set_recipe_page.set(current + 1);
                                        trigger_recipe_fetch(
                                            recipe_query.get(),
                                            current + 1,
                                            recipe_limit,
                                            set_recipe_loading,
                                            set_recipe_status,
                                            set_recipe_list,
                                            set_recipe_total,
                                            set_recipe_filtered,
                                            set_recipe_loaded,
                                            true,
                                            set_recipe_detail_loading,
                                            set_recipe_detail_status,
                                            set_selected_recipe,
                                        );
                                    }
                                } disabled=move || {
                                    let max = (recipe_filtered.get() as f64 / recipe_limit as f64).ceil() as usize;
                                    recipe_loading.get() || max == 0 || recipe_page.get() + 1 >= max
                                }>
                                    "Next"
                                </button>
                            </div>
                        </div>
                        <div class="recipe-detail">
                            <div class="status">{move || recipe_detail_status.get()}</div>
                            <Show
                                when=move || selected_recipe.get().is_some()
                                fallback=move || view! {
                                    <div class="recipe-empty">
                                        <strong>"Select a recipe"</strong>
                                        <p>"Choose a recipe from the list to view ingredients and costing."</p>
                                    </div>
                                }
                            >
                                {move || selected_recipe.with(|opt| {
                                    opt.as_ref().map(|detail| {
                                        let detail = detail.clone();
                                        let instruction_lines = detail
                                            .instructions
                                            .lines()
                                            .enumerate()
                                            .map(|(idx, line)| (idx, line.trim_end().to_string()))
                                            .filter(|(_, line)| !line.is_empty())
                                            .collect::<Vec<(usize, String)>>();
                                        let has_instructions = !instruction_lines.is_empty();
                                        let instruction_lines = Arc::new(instruction_lines);
                                        let recipe_id_for_export = detail.recipe_id;
                                        let recipe_id_for_edit = detail.recipe_id;
                                        let edit_init_name = detail.name.clone();
                                        let edit_init_instructions = detail.instructions.clone();
                                        let recipe_name_for_file = detail.name.chars()
                                            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
                                            .collect::<String>()
                                            .replace(' ', "_");
                                        let pdf_name = format!("{}.pdf", recipe_name_for_file);
                                        let docx_name = format!("{}.docx", recipe_name_for_file);
                                        view! {
                                            <div>
                                                <div class="recipe-title">{detail.name}</div>
                                                <div class="recipe-summary">
                                                    <span>{format!("{} items", detail.item_count)}</span>
                                                    <span>
                                                        {if detail.missing_costs > 0 {
                                                            format!("Cost incomplete ({} missing)", detail.missing_costs)
                                                        } else {
                                                            format!("Total cost {}", format_money(detail.total_cost))
                                                        }}
                                                    </span>
                                                </div>
                                                <Show when={
                                                    let has = !detail.allergens.is_empty();
                                                    move || has
                                                }>
                                                    <div class="allergen-notice">
                                                        <span class="allergen-icon">{"\u{26A0}"}</span>
                                                        <strong>"Allergen Notice: "</strong>
                                                        <span class="allergen-list">
                                                            {detail.allergens.join(", ")}
                                                        </span>
                                                    </div>
                                                </Show>
                                                <div class="row" style="margin: 10px 0; gap: 8px;">
                                                    <button class="button tiny" on:click=move |_| {
                                                        let rid = recipe_id_for_export;
                                                        let name = pdf_name.clone();
                                                        trigger_save_dialog_and_export(
                                                            "Save Recipe PDF",
                                                            &name,
                                                            "PDF",
                                                            "pdf",
                                                            set_export_status,
                                                            move |path| ("export_recipe_pdf".to_string(),
                                                                to_value(&ExportRecipeArgs { recipe_id: rid, output_path: path }).unwrap()),
                                                        );
                                                    }>"Export PDF"</button>
                                                    <button class="button tiny secondary" on:click=move |_| {
                                                        let rid = recipe_id_for_export;
                                                        let name = docx_name.clone();
                                                        trigger_save_dialog_and_export(
                                                            "Save Recipe Word Document",
                                                            &name,
                                                            "Word Document",
                                                            "docx",
                                                            set_export_status,
                                                            move |path| ("export_recipe_docx".to_string(),
                                                                to_value(&ExportRecipeArgs { recipe_id: rid, output_path: path }).unwrap()),
                                                        );
                                                    }>"Export Word"</button>
                                                </div>
                                                <Show when=move || !export_status.get().is_empty()>
                                                    <div class="status">{move || export_status.get()}</div>
                                                </Show>
                                                <div class="detail-block" style="margin-top: 14px;">
                                                    <div class="row" style="align-items: center; gap: 8px;">
                                                        <strong>"Edit Recipe"</strong>
                                                        <button class="button tiny secondary" on:click={
                                                            let init_n = edit_init_name.clone();
                                                            let init_i = edit_init_instructions.clone();
                                                            move |_| {
                                                                if !edit_recipe_editing.get() {
                                                                    set_edit_recipe_name.set(init_n.clone());
                                                                    set_edit_recipe_instructions.set(init_i.clone());
                                                                }
                                                                set_edit_recipe_editing.set(!edit_recipe_editing.get());
                                                                set_edit_recipe_msg.set(String::new());
                                                            }
                                                        }>{move || if edit_recipe_editing.get() { "Cancel" } else { "Edit" }}</button>
                                                    </div>
                                                    <Show when=move || edit_recipe_editing.get()>
                                                        <div style="margin-top: 10px;">
                                                            <div class="input">
                                                                <label>"Name"</label>
                                                                <input
                                                                    type="text"
                                                                    prop:value=move || edit_recipe_name.get()
                                                                    on:input=move |ev| {
                                                                        set_edit_recipe_name.set(event_target_value(&ev));
                                                                    }
                                                                />
                                                            </div>
                                                            <div class="input" style="margin-top: 8px;">
                                                                <label>"Instructions"</label>
                                                                <textarea
                                                                    rows="8"
                                                                    style="width: 100%; font-family: inherit; font-size: 14px; padding: 8px; border-radius: 8px; border: 1px solid var(--border);"
                                                                    prop:value=move || edit_recipe_instructions.get()
                                                                    on:input=move |ev| {
                                                                        set_edit_recipe_instructions.set(event_target_value(&ev));
                                                                    }
                                                                />
                                                            </div>
                                                            <div class="row" style="margin-top: 10px; gap: 8px;">
                                                                <button class="button tiny" on:click=move |_| {
                                                                    save_recipe_edit(recipe_id_for_edit);
                                                                }>"Save Changes"</button>
                                                            </div>
                                                            <Show when=move || !edit_recipe_msg.get().is_empty()>
                                                                <div class="status" style="margin-top: 6px;">{move || edit_recipe_msg.get()}</div>
                                                            </Show>
                                                        </div>
                                                    </Show>
                                                </div>
                                                <div class="recipe-instructions">
                                                    <strong>"Instructions"</strong>
                                                    <Show
                                                        when=move || has_instructions
                                                        fallback=move || view! { <p>"No instructions imported."</p> }
                                                    >
                                                        <div class="instruction-lines">
                                                            <For
                                                                each={
                                                                    let instruction_lines = instruction_lines.clone();
                                                                    move || instruction_lines.as_ref().clone()
                                                                }
                                                                key=|(idx, _)| *idx
                                                                children=move |(_, line)| view! {
                                                                    <p class="instruction-line">{line}</p>
                                                                }
                                                            />
                                                        </div>
                                                    </Show>
                                                </div>
                                                <div class="recipe-ingredients">
                                                    <div
                                                        class="recipe-ingredients-header"
                                                        class:recipe-ingredients-8=move || edit_recipe_editing.get()
                                                    >
                                                        <span>"Item"</span>
                                                        <span>"Qty"</span>
                                                        <span>"Unit"</span>
                                                        <span>"Purch Unit"</span>
                                                        <span>"Price"</span>
                                                        <span>"Cost"</span>
                                                        <span>"Status"</span>
                                                        <Show when=move || edit_recipe_editing.get()>
                                                            <span></span>
                                                        </Show>
                                                    </div>
                                                    <For
                                                        each=move || detail.ingredients.clone()
                                                        key=|item| item.recp_item_id
                                                        children=move |item| {
                                                            let recp_item_id = item.recp_item_id;
                                                            let rid = recipe_id_for_edit;
                                                            view! {
                                                            <div
                                                                class="recipe-ingredients-row"
                                                                class:recipe-ingredients-8=move || edit_recipe_editing.get()
                                                            >
                                                                <span>{item.item_name}</span>
                                                                <span>{item.qty.map(|q| format!("{:.3}", q)).unwrap_or_else(|| "-".to_string())}</span>
                                                                <span>{item.unit_name}</span>
                                                                <span>{item.purch_unit_name}</span>
                                                                <span>{item.price.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                                <span>{item.extended_cost.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                                <span>{item.cost_status}</span>
                                                                <Show when=move || edit_recipe_editing.get()>
                                                                    <button
                                                                        class="button tiny danger"
                                                                        on:click=move |_| {
                                                                            delete_recipe_ingredient(rid, recp_item_id);
                                                                        }
                                                                    >"Remove"</button>
                                                                </Show>
                                                            </div>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            </div>
                                        }
                                    })
                                })}
                            </Show>
                            <Show when=move || recipe_detail_loading.get()>
                                <div class="status">"Loading recipe details..."</div>
                            </Show>
                        </div>
                    </div>
                </Show>

                <Show when=move || active_panel.get() == "vendors">
                    <div class="panel">
                        <div class="row">
                            <div class="input">
                                <label>"Search vendors"</label>
                                <input
                                    type="text"
                                    prop:value=vendor_query
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_vendor_query.set(value);
                                    }
                                />
                            </div>
                            <div class="input" style="align-self: end;">
                                <div class="row">
                                    <button class="button" on:click=move |_| {
                                        set_vendor_page.set(0);
                                        trigger_vendor_fetch(
                                            vendor_query.get(),
                                            0,
                                            vendor_limit,
                                            set_vendor_loading,
                                            set_vendor_status,
                                            set_vendor_list,
                                            set_vendor_total,
                                            set_vendor_filtered,
                                            set_vendor_loaded,
                                        );
                                    } disabled=move || vendor_loading.get()>
                                        {move || if vendor_loading.get() { "Searching..." } else { "Search" }}
                                    </button>
                                    <button class="button secondary" on:click=move |_| {
                                        set_vendor_query.set(String::new());
                                        set_vendor_page.set(0);
                                        trigger_vendor_fetch(
                                            String::new(),
                                            0,
                                            vendor_limit,
                                            set_vendor_loading,
                                            set_vendor_status,
                                            set_vendor_list,
                                            set_vendor_total,
                                            set_vendor_filtered,
                                            set_vendor_loaded,
                                        );
                                    }>
                                        "Reset"
                                    </button>
                                </div>
                            </div>
                        </div>
                        <div class="status">{move || vendor_status.get()}</div>
                    </div>

                    <div class="panel recipe-grid">
                        <div class="recipe-list">
                            <div class="recipe-list-header">
                                <span>"Vendors"</span>
                                <span>{move || format!(
                                    "{} / {}",
                                    vendor_filtered.get(),
                                    vendor_total.get()
                                )}</span>
                            </div>
                            <div class="recipe-list-body">
                                <For
                                    each=move || vendor_list.get()
                                    key=|item| item.vendor_id
                                    children=move |item| {
                                        let vendor_id = item.vendor_id;
                                        view! {
                                            <div
                                                class="recipe-row"
                                                class:selected=move || selected_vendor.get().map(|v| v.vendor_id == vendor_id).unwrap_or(false)
                                                on:click=move |_| {
                                                    set_vendor_merge_source_id.set(vendor_id.to_string());
                                                    trigger_vendor_detail_fetch(
                                                        vendor_id,
                                                        set_vendor_detail_loading,
                                                        set_vendor_detail_status,
                                                        set_selected_vendor,
                                                    );
                                                }
                                            >
                                                <div class="recipe-name">{item.name}</div>
                                                <div class="recipe-meta">{format!("{} priced items", item.price_items)}</div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            <div class="row" style="margin-top: 12px;">
                                <button class="button secondary" on:click=move |_| {
                                    let current = vendor_page.get();
                                    if current > 0 {
                                        set_vendor_page.set(current - 1);
                                        trigger_vendor_fetch(
                                            vendor_query.get(),
                                            current - 1,
                                            vendor_limit,
                                            set_vendor_loading,
                                            set_vendor_status,
                                            set_vendor_list,
                                            set_vendor_total,
                                            set_vendor_filtered,
                                            set_vendor_loaded,
                                        );
                                    }
                                } disabled=move || vendor_page.get() == 0 || vendor_loading.get()>
                                    "Previous"
                                </button>
                                <button class="button" on:click=move |_| {
                                    let current = vendor_page.get();
                                    let max = (vendor_filtered.get() as f64 / vendor_limit as f64).ceil() as usize;
                                    if current + 1 < max {
                                        set_vendor_page.set(current + 1);
                                        trigger_vendor_fetch(
                                            vendor_query.get(),
                                            current + 1,
                                            vendor_limit,
                                            set_vendor_loading,
                                            set_vendor_status,
                                            set_vendor_list,
                                            set_vendor_total,
                                            set_vendor_filtered,
                                            set_vendor_loaded,
                                        );
                                    }
                                } disabled=move || {
                                    let max = (vendor_filtered.get() as f64 / vendor_limit as f64).ceil() as usize;
                                    vendor_loading.get() || max == 0 || vendor_page.get() + 1 >= max
                                }>
                                    "Next"
                                </button>
                            </div>
                        </div>
                        <div class="recipe-detail">
                            <div class="status">{move || vendor_detail_status.get()}</div>
                            <Show
                                when=move || selected_vendor.get().is_some()
                                fallback=move || view! {
                                    <div class="recipe-empty">
                                        <strong>"Select a vendor"</strong>
                                        <p>"Choose a vendor to see price lists and purchasing history."</p>
                                    </div>
                                }
                            >
                                {move || selected_vendor.with(|opt| {
                                    opt.as_ref().map(|detail| {
                                        let detail = detail.clone();
                                        view! {
                                            <div>
                                                <div class="recipe-title">{detail.name}</div>
                                                <div class="recipe-summary">
                                                    <span>{format!("Vendor ID {}", detail.vendor_id)}</span>
                                                    <span>{format!("{} invoices", detail.invoice_count)}</span>
                                                    <span>{format!("{} trans lines", detail.trans_count)}</span>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Merge Vendor IDs"</strong>
                                                    <div class="status">
                                                        "Move pricing, purchasing, and conversion history from one vendor ID into another."
                                                    </div>
                                                    <div class="row" style="margin-top: 10px;">
                                                        <div class="input">
                                                            <label>"Source vendor ID"</label>
                                                            <input
                                                                type="text"
                                                                prop:value=vendor_merge_source_id
                                                                on:input=move |ev| {
                                                                    set_vendor_merge_source_id.set(event_target_value(&ev));
                                                                }
                                                            />
                                                        </div>
                                                        <div class="input">
                                                            <label>"Target vendor"</label>
                                                            <select
                                                                prop:value=vendor_merge_target_id
                                                                on:change=move |ev| {
                                                                    set_vendor_merge_target_id.set(event_target_value(&ev));
                                                                }
                                                            >
                                                                <option value="">"Select target vendor"</option>
                                                                <For
                                                                    each=move || vendor_merge_options.get()
                                                                    key=|v| v.vendor_id
                                                                    children=move |v| view! {
                                                                        <option value={v.vendor_id.to_string()}>{format!("{} - {}", v.vendor_id, v.name)}</option>
                                                                    }
                                                                />
                                                            </select>
                                                        </div>
                                                        <div class="input" style="align-self: end; flex: 0 0 auto;">
                                                            <button class="button secondary" on:click=move |_| merge_vendor_ids()>
                                                                "Merge vendors"
                                                            </button>
                                                        </div>
                                                    </div>
                                                    <div class="status">{move || vendor_merge_status.get()}</div>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Priced Items"</strong>
                                                    <div class="data-table">
                                                        <div class="data-header data-cols-4">
                                                            <span>"Item"</span>
                                                            <span>"Item ID"</span>
                                                            <span>"Price"</span>
                                                            <span>"Pack"</span>
                                                        </div>
                                                        <For
                                                            each=move || detail.price_items.clone()
                                                            key=|item| item.item_id
                                                            children=move |item| view! {
                                                                <div class="data-row data-cols-4">
                                                                    <span>{item.item_name}</span>
                                                                    <span>{item.item_id}</span>
                                                                    <span>{item.price.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                                    <span>{item.pack}</span>
                                                                </div>
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })
                                })}
                            </Show>
                            <Show when=move || vendor_detail_loading.get()>
                                <div class="status">"Loading vendor details..."</div>
                            </Show>
                        </div>
                    </div>
                </Show>

                <Show when=move || active_panel.get() == "conversions">
                    <div class="panel">
                        <div class="row">
                            <div>
                                <strong>"Conversion Overview"</strong>
                                <div class="status">
                                    {move || format!(
                                        "Suggestions: {} | Safe: {} | Todo: {} | Local conversions: {}",
                                        conversion_overview.get().suggestions,
                                        conversion_overview.get().suggestions_safe,
                                        conversion_overview.get().todo,
                                        conversion_overview.get().missing_edges
                                    )}
                                </div>
                            </div>
                        </div>
                        <div class="row" style="margin-top: 12px;">
                            <button
                                class="button secondary"
                                class:active=move || conversion_tab.get() == "suggestions"
                                on:click=move |_| {
                                    set_conversion_tab.set("suggestions".to_string());
                                    set_conversion_page.set(0);
                                    trigger_conversion_suggestions_fetch(
                                        "conv_suggestions".to_string(),
                                        0,
                                        conversion_limit,
                                        set_conversion_loading,
                                        set_conversion_status,
                                        set_conversion_suggestions,
                                        set_conversion_suggestions_total,
                                    );
                                }
                            >
                                "Suggestions"
                            </button>
                            <button
                                class="button secondary"
                                class:active=move || conversion_tab.get() == "safe"
                                on:click=move |_| {
                                    set_conversion_tab.set("safe".to_string());
                                    set_conversion_page.set(0);
                                    trigger_conversion_suggestions_fetch(
                                        "conv_suggestions_safe".to_string(),
                                        0,
                                        conversion_limit,
                                        set_conversion_loading,
                                        set_conversion_status,
                                        set_conversion_suggestions_safe,
                                        set_conversion_suggestions_safe_total,
                                    );
                                }
                            >
                                "Safe"
                            </button>
                            <button
                                class="button secondary"
                                class:active=move || conversion_tab.get() == "todo"
                                on:click=move |_| {
                                    set_conversion_tab.set("todo".to_string());
                                    set_conversion_page.set(0);
                                    trigger_conversion_todo_fetch(
                                        0,
                                        conversion_limit,
                                        set_conversion_loading,
                                        set_conversion_status,
                                        set_conversion_todo,
                                        set_conversion_todo_total,
                                    );
                                }
                            >
                                "Todo"
                            </button>
                            <button
                                class="button secondary"
                                class:active=move || conversion_tab.get() == "missing"
                                on:click=move |_| {
                                    set_conversion_tab.set("missing".to_string());
                                    set_conversion_page.set(0);
                                    trigger_missing_edges_fetch(
                                        0,
                                        conversion_limit,
                                        set_conversion_loading,
                                        set_conversion_status,
                                        set_conversion_missing_edges,
                                        set_conversion_missing_edges_total,
                                    );
                                }
                            >
                                "Local Conversions"
                            </button>
                        </div>
                        <div class="status">{move || conversion_status.get()}</div>
                    </div>

                    <div class="panel">
                        <Show
                            when=move || conversion_tab.get() == "suggestions"
                            fallback=move || ()
                        >
                            <div class="data-table">
                                <div class="data-header data-cols-7">
                                    <span>"Item"</span>
                                    <span>"Units"</span>
                                    <span>"Qty"</span>
                                    <span>"Hits"</span>
                                    <span>"Hops"</span>
                                    <span>"Path"</span>
                                    <span>"Action"</span>
                                </div>
                                <For
                                    each=move || conversion_suggestions.get()
                                    key=|row| (row.item_id, row.unit_id1, row.unit_id2)
                                    children=move |row| view! {
                                        <div class="data-row data-cols-7">
                                            <span>{format!("{} (v{})", row.item_id, row.vendor_id)}</span>
                                            <span>{format!("{} → {}", row.recipe_unit, row.purch_unit)}</span>
                                            <span>{format!("{:.3} / {:.3}", row.qty1, row.qty2)}</span>
                                            <span>{row.hits.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                            <span>{row.hops.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                            <span>{row.path.clone()}</span>
                                            <button class="button tiny" on:click=move |_| approve_conversion(row.clone())>
                                                "Approve"
                                            </button>
                                        </div>
                                    }
                                />
                            </div>
                        </Show>
                        <Show when=move || conversion_tab.get() == "safe">
                            <div class="data-table">
                                <div class="data-header data-cols-7">
                                    <span>"Item"</span>
                                    <span>"Units"</span>
                                    <span>"Qty"</span>
                                    <span>"Hits"</span>
                                    <span>"Hops"</span>
                                    <span>"Path"</span>
                                    <span>"Action"</span>
                                </div>
                                <For
                                    each=move || conversion_suggestions_safe.get()
                                    key=|row| (row.item_id, row.unit_id1, row.unit_id2)
                                    children=move |row| view! {
                                        <div class="data-row data-cols-7">
                                            <span>{format!("{} (v{})", row.item_id, row.vendor_id)}</span>
                                            <span>{format!("{} → {}", row.recipe_unit, row.purch_unit)}</span>
                                            <span>{format!("{:.3} / {:.3}", row.qty1, row.qty2)}</span>
                                            <span>{row.hits.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                            <span>{row.hops.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                            <span>{row.path.clone()}</span>
                                            <button class="button tiny" on:click=move |_| approve_conversion(row.clone())>
                                                "Approve"
                                            </button>
                                        </div>
                                    }
                                />
                            </div>
                        </Show>
                        <Show when=move || conversion_tab.get() == "todo">
                            <div class="data-table">
                                <div class="data-header data-cols-5">
                                    <span>"Item"</span>
                                    <span>"Units"</span>
                                    <span>"Hits"</span>
                                    <span>"Needed"</span>
                                    <span>"Vendor"</span>
                                </div>
                                <For
                                    each=move || conversion_todo.get()
                                    key=|row| (row.item_id, row.recipe_unit_id, row.purch_unit_id)
                                    children=move |row| view! {
                                        <div class="data-row data-cols-5">
                                            <span>{row.item_id}</span>
                                            <span>{format!("{} → {}", row.recipe_unit, row.purch_unit)}</span>
                                            <span>{row.hits.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                            <span>{row.needed}</span>
                                            <span>{row.vendor_id}</span>
                                        </div>
                                    }
                                />
                            </div>
                        </Show>
                        <Show when=move || conversion_tab.get() == "missing">
                            <div class="data-table">
                                <div class="data-header data-cols-8">
                                    <span>"Item"</span>
                                    <span>"Item Name"</span>
                                    <span>"Units"</span>
                                    <span>"Hits"</span>
                                    <span>"Qty (recipe)"</span>
                                    <span>"Qty (purch)"</span>
                                    <span>"Vendor"</span>
                                    <span>"Action"</span>
                                </div>
                                <For
                                    each=move || conversion_missing_edges.get()
                                    key=|row| (row.item_id, row.recipe_unit_id, row.purch_unit_id)
                                    children=move |row| {
                                        let (qty1_val, set_qty1_val) = signal(String::new());
                                        let (qty2_val, set_qty2_val) = signal(String::new());
                                        let (save_status, set_save_status) = signal(String::new());
                                        let row_c = row.clone();
                                        let save_local_conv = move |_| {
                                            let q1: f64 = match qty1_val.get().parse() {
                                                Ok(v) if v > 0.0 => v,
                                                _ => { set_save_status.set("Invalid qty".into()); return; }
                                            };
                                            let q2: f64 = match qty2_val.get().parse() {
                                                Ok(v) if v > 0.0 => v,
                                                _ => { set_save_status.set("Invalid qty".into()); return; }
                                            };
                                            let r = row_c.clone();
                                            set_save_status.set("Saving...".into());
                                            spawn_local(async move {
                                                let args = to_value(&UpsertConvunitArgs {
                                                    item_id: r.item_id,
                                                    vendor_id: r.vendor_id,
                                                    unit_id1: r.recipe_unit_id,
                                                    unit_id2: r.purch_unit_id,
                                                    qty1: q1,
                                                    qty2: q2,
                                                    status: Some(1),
                                                }).unwrap();
                                                match invoke_cmd::<PatchResponse>("upsert_convunit", args).await {
                                                    Ok(resp) => set_save_status.set(resp.message),
                                                    Err(e) => set_save_status.set(format!("Error: {e}")),
                                                }
                                            });
                                        };
                                        view! {
                                            <div class="data-row data-cols-8">
                                                <span>{row.item_id}</span>
                                                <span>{row.item_name}</span>
                                                <span>{format!("{} → {}", row.recipe_unit, row.purch_unit)}</span>
                                                <span>{row.hits.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                                <input
                                                    class="inline-input"
                                                    type="number"
                                                    step="any"
                                                    min="0"
                                                    placeholder="qty"
                                                    prop:value=move || qty1_val.get()
                                                    on:input=move |ev| set_qty1_val.set(event_target_value(&ev))
                                                />
                                                <input
                                                    class="inline-input"
                                                    type="number"
                                                    step="any"
                                                    min="0"
                                                    placeholder="qty"
                                                    prop:value=move || qty2_val.get()
                                                    on:input=move |ev| set_qty2_val.set(event_target_value(&ev))
                                                />
                                                <span>{row.vendor_id}</span>
                                                <span class="action-cell">
                                                    <button class="button tiny" on:click=save_local_conv>"Save"</button>
                                                    <Show when=move || !save_status.get().is_empty()>
                                                        <span class="inline-status">{move || save_status.get()}</span>
                                                    </Show>
                                                </span>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </Show>
                        <div class="row" style="margin-top: 16px;">
                            <button class="button secondary" on:click=move |_| {
                                let current = conversion_page.get();
                                if current > 0 {
                                    set_conversion_page.set(current - 1);
                                    show_conversions();
                                }
                            } disabled=move || conversion_page.get() == 0 || conversion_loading.get()>
                                "Previous"
                            </button>
                            <button class="button" on:click=move |_| {
                                let current = conversion_page.get();
                                let total = match conversion_tab.get().as_str() {
                                    "suggestions" => conversion_suggestions_total.get(),
                                    "safe" => conversion_suggestions_safe_total.get(),
                                    "todo" => conversion_todo_total.get(),
                                    _ => conversion_missing_edges_total.get(),
                                };
                                let max = (total as f64 / conversion_limit as f64).ceil() as usize;
                                if current + 1 < max {
                                    set_conversion_page.set(current + 1);
                                    show_conversions();
                                }
                            } disabled=move || {
                                let total = match conversion_tab.get().as_str() {
                                    "suggestions" => conversion_suggestions_total.get(),
                                    "safe" => conversion_suggestions_safe_total.get(),
                                    "todo" => conversion_todo_total.get(),
                                    _ => conversion_missing_edges_total.get(),
                                };
                                let max = (total as f64 / conversion_limit as f64).ceil() as usize;
                                conversion_loading.get() || max == 0 || conversion_page.get() + 1 >= max
                            }>
                                "Next"
                            </button>
                        </div>
                    </div>
                </Show>

                <Show when=move || active_panel.get() == "purchasing">
                    <div class="panel">
                        <div class="row">
                            <div class="input">
                                <label>"Search invoices"</label>
                                <input
                                    type="text"
                                    prop:value=invoice_query
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_invoice_query.set(value);
                                    }
                                />
                            </div>
                            <div class="input" style="align-self: end;">
                                <div class="row">
                                    <button class="button" on:click=move |_| {
                                        set_invoice_page.set(0);
                                        trigger_invoice_fetch(
                                            invoice_query.get(),
                                            invoice_vendor_filter.get().trim().parse::<i64>().ok(),
                                            invoice_date_from.get(),
                                            invoice_date_to.get(),
                                            0,
                                            invoice_limit,
                                            set_invoice_loading,
                                            set_invoice_status,
                                            set_invoice_list,
                                            set_invoice_total,
                                            set_invoice_filtered,
                                            set_invoice_loaded,
                                        );
                                    } disabled=move || invoice_loading.get()>
                                        {move || if invoice_loading.get() { "Searching..." } else { "Search" }}
                                    </button>
                                    <button class="button secondary" on:click=move |_| {
                                        set_invoice_query.set(String::new());
                                        set_invoice_vendor_filter.set(String::new());
                                        set_invoice_date_from.set(String::new());
                                        set_invoice_date_to.set(String::new());
                                        set_invoice_page.set(0);
                                        trigger_invoice_fetch(
                                            String::new(),
                                            None,
                                            String::new(),
                                            String::new(),
                                            0,
                                            invoice_limit,
                                            set_invoice_loading,
                                            set_invoice_status,
                                            set_invoice_list,
                                            set_invoice_total,
                                            set_invoice_filtered,
                                            set_invoice_loaded,
                                        );
                                    }>
                                        "Reset"
                                    </button>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="input">
                                <label>"Vendor ID"</label>
                                <select
                                    prop:value=invoice_vendor_filter
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_invoice_vendor_filter.set(value);
                                    }
                                >
                                    <option value="">"All vendors"</option>
                                    <For
                                        each=move || invoice_vendor_options.get()
                                        key=|v| v.vendor_id
                                        children=move |v| view! {
                                            <option value={v.vendor_id.to_string()}>{format!("{} - {}", v.vendor_id, v.name)}</option>
                                        }
                                    />
                                </select>
                            </div>
                            <div class="input">
                                <label>"Date from"</label>
                                <input
                                    type="date"
                                    prop:value=invoice_date_from
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_invoice_date_from.set(value);
                                    }
                                />
                            </div>
                            <div class="input">
                                <label>"Date to"</label>
                                <input
                                    type="date"
                                    prop:value=invoice_date_to
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        set_invoice_date_to.set(value);
                                    }
                                />
                            </div>
                            <div class="input" style="align-self: end;">
                                <button class="button secondary" on:click=move |_| export_invoices()>
                                    "Export CSV"
                                </button>
                            </div>
                        </div>
                        <div class="status">{move || invoice_status.get()}</div>
                        <div class="status">{move || invoice_export_status.get()}</div>
                        <div class="status">
                            "Source: SQLite (imported from Invoice.csv + Trans.csv)"
                        </div>
                        <Show when=move || !invoice_export_path.get().is_empty()>
                            <div class="row">
                                <button class="button secondary" on:click=move |_| {
                                    open_path(invoice_export_path.get());
                                }>
                                    "Reveal export file"
                                </button>
                                <div class="status">{move || invoice_export_path.get()}</div>
                            </div>
                        </Show>
                    </div>

                    <div class="panel recipe-grid">
                        <div class="recipe-list">
                            <div class="recipe-list-header">
                                <span>"Invoices"</span>
                                <span>{move || format!(
                                    "{} / {}",
                                    invoice_filtered.get(),
                                    invoice_total.get()
                                )}</span>
                            </div>
                            <div class="recipe-list-body">
                                <For
                                    each=move || invoice_list.get()
                                    key=|item| item.invoice_id
                                    children=move |item| {
                                        let invoice_id = item.invoice_id;
                                        view! {
                                            <div
                                                class="recipe-row"
                                                class:selected=move || selected_invoice.get().map(|r| r.invoice.invoice_id == invoice_id).unwrap_or(false)
                                                on:click=move |_| {
                                                    trigger_invoice_detail_fetch(
                                                        invoice_id,
                                                        set_invoice_detail_loading,
                                                        set_invoice_detail_status,
                                                        set_selected_invoice,
                                                    );
                                                }
                                            >
                                                <div class="recipe-name">{item.invoice_no.clone()}</div>
                                                <div class="recipe-meta">{format!("{} | {}", item.vendor_name, item.invoice_date)}</div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            <div class="row" style="margin-top: 12px;">
                                <button class="button secondary" on:click=move |_| {
                                    let current = invoice_page.get();
                                    if current > 0 {
                                        set_invoice_page.set(current - 1);
                                        trigger_invoice_fetch(
                                            invoice_query.get(),
                                            invoice_vendor_filter.get().trim().parse::<i64>().ok(),
                                            invoice_date_from.get(),
                                            invoice_date_to.get(),
                                            current - 1,
                                            invoice_limit,
                                            set_invoice_loading,
                                            set_invoice_status,
                                            set_invoice_list,
                                            set_invoice_total,
                                            set_invoice_filtered,
                                            set_invoice_loaded,
                                        );
                                    }
                                } disabled=move || invoice_page.get() == 0 || invoice_loading.get()>
                                    "Previous"
                                </button>
                                <button class="button" on:click=move |_| {
                                    let current = invoice_page.get();
                                    let max = (invoice_filtered.get() as f64 / invoice_limit as f64).ceil() as usize;
                                    if current + 1 < max {
                                        set_invoice_page.set(current + 1);
                                        trigger_invoice_fetch(
                                            invoice_query.get(),
                                            invoice_vendor_filter.get().trim().parse::<i64>().ok(),
                                            invoice_date_from.get(),
                                            invoice_date_to.get(),
                                            current + 1,
                                            invoice_limit,
                                            set_invoice_loading,
                                            set_invoice_status,
                                            set_invoice_list,
                                            set_invoice_total,
                                            set_invoice_filtered,
                                            set_invoice_loaded,
                                        );
                                    }
                                } disabled=move || {
                                    let max = (invoice_filtered.get() as f64 / invoice_limit as f64).ceil() as usize;
                                    invoice_loading.get() || max == 0 || invoice_page.get() + 1 >= max
                                }>
                                    "Next"
                                </button>
                            </div>
                        </div>
                        <div class="recipe-detail">
                            <div class="status">{move || invoice_detail_status.get()}</div>
                            <Show
                                when=move || selected_invoice.get().is_some()
                                fallback=move || view! {
                                    <div class="recipe-empty">
                                        <strong>"Select an invoice"</strong>
                                        <p>"Choose an invoice to review transaction lines."</p>
                                    </div>
                                }
                            >
                                {move || selected_invoice.with(|opt| {
                                    opt.as_ref().map(|detail| {
                                        let detail = detail.clone();
                                        view! {
                                            <div>
                                                <div class="recipe-title">{detail.invoice.invoice_no.clone()}</div>
                                                <div class="recipe-summary">
                                                    <span>{detail.invoice.vendor_name.clone()}</span>
                                                    <span>{detail.invoice.invoice_date.clone()}</span>
                                                    <span>{detail.invoice.total.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                    <span>{detail.freight.map(|v| format!("Freight {}", format_money(v))).unwrap_or_else(|| "Freight -".to_string())}</span>
                                                    <span>
                                                        {format!(
                                                            "Line total {}",
                                                            format_money(detail.lines.iter().filter_map(|l| l.ext_cost).sum::<f64>())
                                                        )}
                                                    </span>
                                                </div>
                                                <div class="detail-block">
                                                    <strong>"Lines"</strong>
                                                    <div class="data-table">
                                                        <div class="data-header data-cols-6">
                                                            <span>"Item"</span>
                                                            <span>"Qty"</span>
                                                            <span>"Unit"</span>
                                                            <span>"Price"</span>
                                                            <span>"Ext"</span>
                                                            <span>"Trans ID"</span>
                                                        </div>
                                                        <For
                                                            each=move || detail.lines.clone()
                                                            key=|item| (item.trans_id.unwrap_or(0), item.item_id.unwrap_or(0))
                                                            children=move |item| view! {
                                                                <div class="data-row data-cols-6">
                                                                    <span>{item.item_name}</span>
                                                                    <span>{item.qty.map(|q| format!("{:.3}", q)).unwrap_or_else(|| "-".to_string())}</span>
                                                                    <span>{item.unit_name}</span>
                                                                    <span>{item.price.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                                    <span>{item.ext_cost.map(format_money).unwrap_or_else(|| "-".to_string())}</span>
                                                                    <span>{item.trans_id.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                                                </div>
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })
                                })}
                            </Show>
                            <Show when=move || invoice_detail_loading.get()>
                                <div class="status">"Loading invoice..."</div>
                            </Show>
                        </div>
                    </div>
                </Show>

                <Show when=move || active_panel.get() == "reports">
                    <div class="panel">
                        <div class="row">
                            <div>
                                <strong>"Missing Data Report"</strong>
                                <div class="status">
                                    {move || format!("Rows: {}", missing_data_total.get())}
                                </div>
                            </div>
                        </div>
                        <div class="data-table">
                            <div class="data-header data-cols-5">
                                <span>"Recipe"</span>
                                <span>"Recipe ID"</span>
                                <span>"Missing Costs"</span>
                                <span>"Missing Conversions"</span>
                                <span>"Missing Units"</span>
                            </div>
                            <For
                                each=move || missing_data_rows.get()
                                key=|row| row.recipe_id
                                children=move |row| view! {
                                    <div class="data-row data-cols-5">
                                        <span>{row.recipe_name}</span>
                                        <span>{row.recipe_id}</span>
                                        <span>{row.missing_a.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                        <span>{row.missing_b.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                        <span>{row.missing_c.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                    </div>
                                }
                            />
                        </div>
                    </div>

                    <div class="panel">
                        <div class="row">
                            <div>
                                <strong>"Missing Purchase Units"</strong>
                                <div class="status">
                                    {move || format!("Rows: {}", missing_purch_total.get())}
                                </div>
                            </div>
                        </div>
                        <div class="row" style="margin-top: 10px;">
                            <div class="input">
                                <label>"Item ID"</label>
                                <input
                                    type="text"
                                    prop:value=report_purch_item_id
                                    on:input=move |ev| {
                                        set_report_purch_item_id.set(event_target_value(&ev));
                                    }
                                />
                            </div>
                            <div class="input">
                                <label>"Purchase Unit"</label>
                                <select
                                    prop:value=purch_unit_selected_id
                                    on:change=move |ev| {
                                        set_purch_unit_selected_id.set(event_target_value(&ev));
                                    }
                                >
                                    <option value="">"Select unit"</option>
                                    <For
                                        each=move || unit_options.get()
                                        key=|unit| unit.unit_id
                                        children=move |unit| view! {
                                            <option value={unit.unit_id.to_string()}>{format!("{} - {}", unit.unit_id, unit.sing)}</option>
                                        }
                                    />
                                </select>
                            </div>
                            <div class="input" style="flex: 0 0 150px;">
                                <label>"Default"</label>
                                <label style="display: inline-flex; gap: 8px; align-items: center;">
                                    <input
                                        type="checkbox"
                                        prop:checked=purch_unit_set_default
                                        on:change=move |ev| {
                                            set_purch_unit_set_default.set(event_target_checked(&ev));
                                        }
                                    />
                                    <span>"Set default"</span>
                                </label>
                            </div>
                            <div class="input" style="align-self: end; flex: 0 0 auto;">
                                <button class="button" on:click=move |_| {
                                    match report_purch_item_id.get().trim().parse::<i64>() {
                                        Ok(item_id) => assign_purch_unit(item_id),
                                        Err(_) => set_purch_unit_status.set("Enter a valid item ID".to_string()),
                                    }
                                }>
                                    "Assign unit"
                                </button>
                            </div>
                        </div>
                        <div class="status">{move || purch_unit_status.get()}</div>
                        <div class="data-table">
                            <div class="data-header data-cols-3">
                                <span>"Item"</span>
                                <span>"Item ID"</span>
                                <span>"Usage Count"</span>
                            </div>
                            <For
                                each=move || missing_purch_rows.get()
                                key=|row| row.item_id
                                children=move |row| {
                                    let row_item_id = row.item_id;
                                    view! {
                                        <div class="data-row data-cols-3" on:click=move |_| {
                                            set_report_purch_item_id.set(row_item_id.to_string());
                                        }>
                                            <span>{row.item_name}</span>
                                            <span>{row.item_id}</span>
                                            <span>{row.usage_count.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())}</span>
                                        </div>
                                    }
                                }
                            />
                        </div>
                        <div class="status">{move || report_status.get()}</div>
                    </div>
                </Show>

                // ── Food Cost Panel ──
                <Show when=move || active_panel.get() == "foodcost">
                    <div class="panel">
                        <div class="row" style="align-items: center; gap: 12px; margin-bottom: 12px;">
                            <div class="input" style="flex: 1;">
                                <label>"Dish name (optional)"</label>
                                <input
                                    type="text"
                                    placeholder="e.g. Grilled Salmon Plate"
                                    prop:value=fc_dish_name
                                    on:input=move |ev| set_fc_dish_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="input" style="max-width: 160px;">
                                <label>"Target food cost %"</label>
                                <input
                                    type="number"
                                    min="1"
                                    max="100"
                                    step="0.5"
                                    prop:value=move || format!("{}", fc_target_pct.get())
                                    on:input=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                            set_fc_target_pct.set(v);
                                        }
                                    }
                                />
                            </div>
                        </div>

                        <div class="fc-add-row">
                            <div class="input" style="flex: 2;">
                                <label>"Ingredient"</label>
                                <select
                                    prop:value=fc_add_item_id
                                    on:change=move |ev| set_fc_add_item_id.set(event_target_value(&ev))
                                >
                                    <option value="">"-- Select item --"</option>
                                    <For
                                        each=move || fc_item_options.get()
                                        key=|item| item.item_id
                                        children=move |item| {
                                            let val = item.item_id.to_string();
                                            view! { <option value={val.clone()}>{format!("{} ({})", item.name, item.item_id)}</option> }
                                        }
                                    />
                                </select>
                            </div>
                            <div class="input" style="flex: 1;">
                                <label>"Unit"</label>
                                <select
                                    prop:value=fc_add_unit_id
                                    on:change=move |ev| set_fc_add_unit_id.set(event_target_value(&ev))
                                >
                                    <option value="">"-- Unit --"</option>
                                    <For
                                        each=move || fc_unit_options.get()
                                        key=|u| u.unit_id
                                        children=move |u| {
                                            let val = u.unit_id.to_string();
                                            view! { <option value={val.clone()}>{u.sing.clone()}</option> }
                                        }
                                    />
                                </select>
                            </div>
                            <div class="input" style="max-width: 100px;">
                                <label>"Qty"</label>
                                <input
                                    type="number"
                                    step="0.01"
                                    min="0"
                                    placeholder="0.00"
                                    prop:value=fc_add_qty
                                    on:input=move |ev| set_fc_add_qty.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="input">
                                <label style="visibility: hidden;">{"\u{00A0}"}</label>
                                <button class="button secondary" on:click=move |_| fc_add_line()>
                                    "+ Add"
                                </button>
                            </div>
                        </div>

                        // ingredient lines table
                        <Show when=move || !fc_lines.get().is_empty()>
                            <div class="data-table" style="margin-top: 12px;">
                                <div class="data-header data-cols-4">
                                    <span>"Ingredient"</span>
                                    <span>"Unit"</span>
                                    <span>"Qty"</span>
                                    <span></span>
                                </div>
                                <For
                                    each=move || {
                                        fc_lines.get().iter().enumerate().map(|(i, l)| (i, l.clone())).collect::<Vec<_>>()
                                    }
                                    key=|(i, _)| *i
                                    children=move |(idx, line)| {
                                        let item_name = fc_item_options.get().iter()
                                            .find(|it| it.item_id == line.item_id)
                                            .map(|it| it.name.clone())
                                            .unwrap_or_else(|| format!("Item {}", line.item_id));
                                        let unit_name = line.unit_id
                                            .and_then(|uid| fc_unit_options.get().iter().find(|u| u.unit_id == uid).map(|u| u.sing.clone()))
                                            .unwrap_or_else(|| "-".to_string());
                                        view! {
                                            <div class="data-row data-cols-4">
                                                <span>{item_name}</span>
                                                <span>{unit_name}</span>
                                                <span>{format!("{:.2}", line.qty)}</span>
                                                <span>
                                                    <button class="button-link danger" on:click=move |_| fc_remove_line(idx)>
                                                        "\u{2715}"
                                                    </button>
                                                </span>
                                            </div>
                                        }
                                    }
                                />
                            </div>

                            <div class="row" style="margin-top: 12px; gap: 8px;">
                                <button class="button" on:click=move |_| fc_calculate() disabled=move || fc_loading.get()>
                                    {move || if fc_loading.get() { "Calculating\u{2026}" } else { "Calculate Cost" }}
                                </button>
                                <button class="button secondary" on:click=move |_| fc_clear()>
                                    "Clear All"
                                </button>
                            </div>
                        </Show>

                        <div class="status">{move || fc_status.get()}</div>

                        // results
                        <Show when=move || fc_result.get().is_some()>
                            {move || {
                                let result = fc_result.get().unwrap_or_default();
                                let total = result.total_cost;
                                let missing = result.missing_costs;
                                let pct = fc_target_pct.get();
                                let menu_price = if pct > 0.0 { total / (pct / 100.0) } else { 0.0 };
                                let dish = fc_dish_name.get();
                                let dish_label = if dish.trim().is_empty() { "This dish".to_string() } else { dish };

                                view! {
                                    <div class="fc-results">
                                        <div class="fc-summary-cards">
                                            <div class="card fc-card">
                                                <h4>"Total Ingredient Cost"</h4>
                                                <p class="fc-big-number">{format_money(total)}</p>
                                            </div>
                                            <div class="card fc-card fc-card-highlight">
                                                <h4>"Suggested Menu Price"</h4>
                                                <p class="fc-big-number">{format_money(menu_price)}</p>
                                                <p class="fc-sub">{format!("at {}% food cost", pct)}</p>
                                            </div>
                                            <div class="card fc-card">
                                                <h4>"Profit per Plate"</h4>
                                                <p class="fc-big-number">{format_money(menu_price - total)}</p>
                                            </div>
                                        </div>

                                        <Show when=move || { missing > 0 }>
                                            <div class="fc-warning">
                                                {format!("\u{26A0} {} ingredient(s) could not be costed. The total may be understated.", missing)}
                                            </div>
                                        </Show>

                                        <div class="fc-detail-label">
                                            {format!("\u{1F4CB} {} \u{2014} ingredient breakdown", dish_label)}
                                        </div>
                                        <div class="data-table">
                                            <div class="data-header data-cols-5">
                                                <span>"Ingredient"</span>
                                                <span>"Qty"</span>
                                                <span>"Unit Price"</span>
                                                <span>"Line Cost"</span>
                                                <span>"Status"</span>
                                            </div>
                                            <For
                                                each=move || result.lines.clone()
                                                key=|line| line.item_id
                                                children=move |line| {
                                                    let status_class = if line.cost_status == "OK" { "fc-status-ok" } else { "fc-status-warn" };
                                                    view! {
                                                        <div class="data-row data-cols-5">
                                                            <span>{format!("{} ({})", line.item_name, line.unit_name)}</span>
                                                            <span>{format!("{:.2}", line.qty)}</span>
                                                            <span>{line.price.map(|p| format_money(p)).unwrap_or_else(|| "\u{2014}".to_string())}</span>
                                                            <span>{line.extended_cost.map(|c| format_money(c)).unwrap_or_else(|| "\u{2014}".to_string())}</span>
                                                            <span class={status_class}>{line.cost_status.clone()}</span>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    </div>
                                }
                            }}
                        </Show>
                    </div>
                </Show>
                <Show when=move || active_panel.get() == "settings">
                    <div class="panel">
                        <div class="settings-grid">
                            <div class="settings-section">
                                <h3>"Company Information"</h3>
                                <div class="input">
                                    <label>"Restaurant / Company Name"</label>
                                    <input
                                        type="text"
                                        placeholder="e.g. The Golden Fork"
                                        prop:value=move || settings_company.get()
                                        on:input=move |ev| {
                                            set_settings_company.set(event_target_value(&ev));
                                        }
                                    />
                                </div>
                                <div class="input" style="margin-top: 14px;">
                                    <label>"Company Logo"</label>
                                    <Show
                                        when=move || !settings_logo_path.get().is_empty()
                                        fallback=move || view! {
                                            <p class="settings-hint">"No logo uploaded yet."</p>
                                        }
                                    >
                                        <div class="settings-logo-preview">
                                            <img
                                                src={move || {
                                                    let p = settings_logo_path.get();
                                                    if p.is_empty() {
                                                        String::new()
                                                    } else {
                                                        format!("asset://localhost/{}", p)
                                                    }
                                                }}
                                                alt="Company logo"
                                            />
                                            <button
                                                class="button tiny danger"
                                                on:click=move |_| remove_logo_action()
                                            >"Remove"</button>
                                        </div>
                                    </Show>
                                    <div class="row" style="margin-top: 8px; gap: 8px;">
                                        <input
                                            type="text"
                                            placeholder="Path to logo file (PNG, JPG, SVG...)"
                                            style="flex: 1;"
                                            prop:value=move || settings_logo_upload_path.get()
                                            on:input=move |ev| {
                                                set_settings_logo_upload_path.set(event_target_value(&ev));
                                            }
                                        />
                                        <button
                                            class="button tiny"
                                            on:click=move |_| upload_logo_action()
                                        >"Upload"</button>
                                    </div>
                                </div>
                            </div>
                            <div class="settings-section">
                                <h3>"Food Service Category"</h3>
                                <div class="settings-radio-group">
                                    {[
                                        ("full_service", "Full Service"),
                                        ("quick_service", "Quick Service"),
                                        ("fast_casual", "Fast Casual"),
                                        ("self_service", "Self-Service"),
                                        ("delivery_takeaway", "Delivery & Takeaway"),
                                    ].into_iter().map(|(value, label)| {
                                        let value_owned = value.to_string();
                                        let value_check = value.to_string();
                                        view! {
                                            <label class="settings-radio">
                                                <input
                                                    type="radio"
                                                    name="service_category"
                                                    value={value_owned.clone()}
                                                    prop:checked=move || settings_service_cat.get() == value_check
                                                    on:change={
                                                        let v = value_owned.clone();
                                                        move |_| set_settings_service_cat.set(v.clone())
                                                    }
                                                />
                                                {label}
                                            </label>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                            <div class="settings-section">
                                <h3>"Operation Size"</h3>
                                <div class="settings-radio-group">
                                    {[
                                        ("small_business", "Small Business (5\u{2013}50 employees)"),
                                        ("franchise", "Franchise"),
                                        ("chain", "Chain"),
                                    ].into_iter().map(|(value, label)| {
                                        let value_owned = value.to_string();
                                        let value_check = value.to_string();
                                        view! {
                                            <label class="settings-radio">
                                                <input
                                                    type="radio"
                                                    name="operation_size"
                                                    value={value_owned.clone()}
                                                    prop:checked=move || settings_op_size.get() == value_check
                                                    on:change={
                                                        let v = value_owned.clone();
                                                        move |_| set_settings_op_size.set(v.clone())
                                                    }
                                                />
                                                {label}
                                            </label>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </div>
                        <div class="row" style="margin-top: 20px; gap: 8px;">
                            <button
                                class="button"
                                on:click=move |_| save_settings_action()
                            >"Save Settings"</button>
                        </div>
                        <Show when=move || !settings_status.get().is_empty()>
                            <div class="status" style="margin-top: 10px;">{move || settings_status.get()}</div>
                        </Show>
                    </div>
                </Show>

                // ── FDA Guidelines Panel ──
                <Show when=move || active_panel.get() == "fda">
                    <div class="panel">
                        <div class="fda-toolbar">
                            <button class="button tiny" on:click=move |_| export_fda_pdf()>"Export PDF"</button>
                        </div>
                        <div class="fda-grid">

                            // ── Temperature Control ──
                            <div class="fda-card">
                                <h3>"Temperature Control"</h3>
                                <table class="fda-table">
                                    <thead><tr>
                                        <th>"Zone"</th><th>"Range"</th><th>"Guideline"</th>
                                    </tr></thead>
                                    <tbody>
                                        <tr><td>"Danger Zone"</td><td>"41 °F – 135 °F (5 °C – 57 °C)"</td><td>"Food must not remain in this range for more than 4 hours total."</td></tr>
                                        <tr><td>"Cold holding"</td><td>"≤ 41 °F (5 °C)"</td><td>"Keep cold foods at or below 41 °F at all times."</td></tr>
                                        <tr><td>"Hot holding"</td><td>"≥ 135 °F (57 °C)"</td><td>"Keep hot foods at or above 135 °F."</td></tr>
                                        <tr><td>"Receiving"</td><td>"≤ 41 °F / frozen solid"</td><td>"Reject deliveries above 41 °F (refrigerated) or partially thawed (frozen)."</td></tr>
                                    </tbody>
                                </table>
                            </div>

                            // ── Cooking Temperatures ──
                            <div class="fda-card">
                                <h3>"Minimum Cooking Temperatures"</h3>
                                <table class="fda-table">
                                    <thead><tr>
                                        <th>"Food"</th><th>"Internal Temp"</th><th>"Hold Time"</th>
                                    </tr></thead>
                                    <tbody>
                                        <tr><td>"Poultry (chicken, turkey, duck)"</td><td>"165 °F (74 °C)"</td><td>"Instantaneous"</td></tr>
                                        <tr><td>"Ground meat (beef, pork, lamb)"</td><td>"155 °F (68 °C)"</td><td>"17 seconds"</td></tr>
                                        <tr><td>"Seafood, steaks, chops, eggs for service"</td><td>"145 °F (63 °C)"</td><td>"15 seconds"</td></tr>
                                        <tr><td>"Roasts (beef, pork, lamb)"</td><td>"145 °F (63 °C)"</td><td>"4 minutes"</td></tr>
                                        <tr><td>"Fruits, vegetables, grains (hot holding)"</td><td>"135 °F (57 °C)"</td><td>"Instantaneous"</td></tr>
                                        <tr><td>"Reheated leftovers"</td><td>"165 °F (74 °C)"</td><td>"Within 2 hours"</td></tr>
                                    </tbody>
                                </table>
                            </div>

                            // ── Cooling Requirements ──
                            <div class="fda-card">
                                <h3>"Two-Stage Cooling"</h3>
                                <table class="fda-table">
                                    <thead><tr>
                                        <th>"Stage"</th><th>"Target"</th><th>"Time Limit"</th>
                                    </tr></thead>
                                    <tbody>
                                        <tr><td>"Stage 1"</td><td>"135 °F → 70 °F"</td><td>"Within 2 hours"</td></tr>
                                        <tr><td>"Stage 2"</td><td>"70 °F → 41 °F"</td><td>"Within 4 hours (6 hours total)"</td></tr>
                                    </tbody>
                                </table>
                                <p class="fda-note">"If food does not reach 70 °F within 2 hours, it must be reheated to 165 °F and the cooling process restarted."</p>
                            </div>

                            // ── Thawing Methods ──
                            <div class="fda-card">
                                <h3>"Approved Thawing Methods"</h3>
                                <ul class="fda-list">
                                    <li>"Refrigerator thawing — at 41 °F or below"</li>
                                    <li>"Cold running water — submerged, ≤ 70 °F, used within 4 hours"</li>
                                    <li>"Microwave — only if cooked immediately after"</li>
                                    <li>"Cooking from frozen — as part of the cooking process"</li>
                                </ul>
                                <p class="fda-note">"Never thaw food at room temperature on a counter."</p>
                            </div>

                            // ── Handwashing ──
                            <div class="fda-card">
                                <h3>"Handwashing"</h3>
                                <ul class="fda-list">
                                    <li>"Wet hands with warm water (≥ 100 °F / 38 °C)"</li>
                                    <li>"Apply soap and scrub for at least 20 seconds"</li>
                                    <li>"Rinse and dry with single-use towel or air dryer"</li>
                                </ul>
                                <p class="fda-note">"Required: before handling food, after touching raw meat, after using the restroom, after sneezing/coughing, after handling trash."</p>
                            </div>

                            // ── Cross-Contamination ──
                            <div class="fda-card">
                                <h3>"Cross-Contamination Prevention"</h3>
                                <ul class="fda-list">
                                    <li>"Store raw meats below ready-to-eat foods in the cooler"</li>
                                    <li>"Cooler order (top → bottom): ready-to-eat, seafood, whole cuts, ground meat, poultry"</li>
                                    <li>"Use separate cutting boards and utensils for raw and cooked foods"</li>
                                    <li>"Sanitize surfaces between tasks — use approved sanitizer at correct concentration"</li>
                                </ul>
                            </div>

                            // ── Storage Shelf-Life ──
                            <div class="fda-card">
                                <h3>"Maximum Cold Storage (at 41 °F)"</h3>
                                <table class="fda-table">
                                    <thead><tr>
                                        <th>"Item"</th><th>"Max Days"</th>
                                    </tr></thead>
                                    <tbody>
                                        <tr><td>"Fresh poultry, ground meat, fish"</td><td>"1 – 2 days"</td></tr>
                                        <tr><td>"Fresh steaks, chops, roasts"</td><td>"3 – 5 days"</td></tr>
                                        <tr><td>"Cooked leftovers"</td><td>"7 days (date-mark required)"</td></tr>
                                        <tr><td>"Deli meats (opened)"</td><td>"3 – 5 days"</td></tr>
                                        <tr><td>"Eggs (shell)"</td><td>"3 – 5 weeks"</td></tr>
                                    </tbody>
                                </table>
                                <p class="fda-note">"All ready-to-eat TCS foods held longer than 24 hours must be date-marked."</p>
                            </div>

                            // ── Big 6 Pathogens ──
                            <div class="fda-card">
                                <h3>"Big 6 Foodborne Pathogens"</h3>
                                <table class="fda-table">
                                    <thead><tr>
                                        <th>"Pathogen"</th><th>"Common Sources"</th>
                                    </tr></thead>
                                    <tbody>
                                        <tr><td>"Norovirus"</td><td>"Infected workers, ready-to-eat foods"</td></tr>
                                        <tr><td>"Salmonella Typhi"</td><td>"Beverages, ready-to-eat foods"</td></tr>
                                        <tr><td>"Shigella spp."</td><td>"Ready-to-eat foods, contaminated water"</td></tr>
                                        <tr><td>"E. coli O157:H7"</td><td>"Undercooked beef, unpasteurized juices"</td></tr>
                                        <tr><td>"Hepatitis A"</td><td>"Shellfish, ready-to-eat foods"</td></tr>
                                        <tr><td>"Non-typhoidal Salmonella"</td><td>"Poultry, eggs, produce"</td></tr>
                                    </tbody>
                                </table>
                                <p class="fda-note">"Employees diagnosed with any Big 6 illness must be excluded or restricted per FDA Food Code."</p>
                            </div>

                        </div>
                    </div>
                </Show>
            </main>
        </div>
    }
}
