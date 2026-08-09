use crate::utilities::*;

use std::{collections::HashMap, sync::LazyLock};

use anyhow::{Context, Result, anyhow};

use crate::exceptions::DC_DATA_NO_RESULT_EXCEPTION;
use crate::json;

pub const DCDATA_ID_COL: usize = 0;
pub const DCDATA_NAME_COL: usize = 1;
pub const DCDATA_COMBINING_CLASS_COL: usize = 2;
/// Bidi class column index.
pub const DCDATA_BIDI_CLASS_COL: usize = 3;
pub const DCDATA_CASING_COL: usize = 4;
/// General category column index
pub const DCDATA_TYPE_COL: usize = 5;
/// Script column index.
pub const DCDATA_SCRIPT_COL: usize = 6;
pub const DCDATA_COMPLEX_TRAITS_COL: usize = 7;
pub const DCDATA_DESCRIPTION_COL: usize = 8;

pub const DC_FORMATS_ID_COL: usize = 0;
pub const DC_FORMATS_NAME_COL: usize = 1;
pub const DC_FORMATS_EXTENSION_COL: usize = 2;
pub const DC_FORMATS_IMPORT_SUPPORT_COL: usize = 3;
pub const DC_FORMATS_EXPORT_SUPPORT_COL: usize = 4;
pub const DC_FORMATS_TEST_COVERAGE_COL: usize = 5;
pub const DC_FORMATS_TYPE_COL: usize = 6;
pub const DC_FORMATS_LABEL_COL: usize = 7;
pub const DC_FORMATS_VARIANT_TYPES_COL: usize = 8;
pub const DC_FORMATS_COMMENTS_COL: usize = 9;

pub struct EiteData {
    // Data / datasets
    datasets: Vec<String>,
    datasets_loaded: bool,
    pub data: HashMap<String, Vec<Vec<String>>>, // dataset name -> rows (including header + trailing newline row)
}

impl EiteData {
    pub fn new() -> Result<Self> {
        let mut data = HashMap::new();
        let mut loaded_datasets = Vec::new();

        for dataset_name in list_dc_datasets() {
            let path = format!("{dataset_name}.csv");
            // Try to load the asset
            let dataset_bytes = crate::get_eite_data(&path)
                .with_context(|| format!("asset not found: {path}"))?;
            let mut rdr =
                csv::Reader::from_reader(std::io::Cursor::new(dataset_bytes));
            let mut rows = Vec::new();
            for result in rdr.records() {
                match result {
                    Ok(record) => {
                        // record is a csv::StringRecord, convert to Vec<String>
                        rows.push(
                            record
                                .iter()
                                .map(std::string::ToString::to_string)
                                .collect(),
                        );
                    }
                    Err(e) => {
                        anyhow::bail!(
                            "Error parsing row in dataset {dataset_name}: {e}"
                        );
                    }
                }
            }
            data.insert(dataset_name.to_string(), rows);
            loaded_datasets.push(dataset_name.to_string());
        }

        Ok(Self {
            datasets: loaded_datasets,
            datasets_loaded: true,
            data,
        })
    }

    pub fn json(&self) -> String {
        json!(self.data).to_string()
    }

    /// Returns total rows excluding header.
    fn dc_dataset_length(&self, dataset: &str) -> Result<usize> {
        let rows = self
            .data
            .get(dataset)
            .with_context(|| format!("dataset not loaded: {dataset}"))?;
        Ok(rows.len().saturating_sub(2))
    }

    fn dc_data_get_column(&self, dataset: &str, col_num: usize) -> Result<Vec<String>> {
        let rows = self
            .data
            .get(dataset)
            .with_context(|| format!("dataset not loaded: {dataset}"))?;
        let mut out = Vec::new();
        for row in rows.iter().take(rows.len()) {
            if let Some(v) = row.get(col_num) {
                out.push(v.clone());
            }
        }
        Ok(out)
    }

    /// rowNum is zero-based for content rows (header skipped).
    /// If out of range (beyond trailing sentinel) returns UUID sentinel constant.
    fn dc_data_lookup_by_id(
        &self,
        dataset: &str,
        row_num: usize,
        field_num: usize,
    ) -> Result<String> {
        let rows = self
            .data
            .get(dataset)
            .ok_or_else(|| anyhow!("dataset not loaded: {dataset}"))?;

        if row_num >= rows.len() {
            return Err(anyhow!(DC_DATA_NO_RESULT_EXCEPTION.to_string()));
        }

        let row = rows
            .get(row_num)
            .ok_or_else(|| anyhow!("index out of bounds"))?;
        let value = row
            .get(field_num)
            .cloned()
            .ok_or_else(|| anyhow!(DC_DATA_NO_RESULT_EXCEPTION.to_string()))?;

        Ok(value)
    }

    /// Returns first match or sentinel UUID if none.
    fn dc_data_lookup_by_value(
        &self,
        dataset: &str,
        filter_field: usize,
        filter_value: &str,
        desired_field: usize,
    ) -> Result<String> {
        let rows = self
            .data
            .get(dataset)
            .ok_or_else(|| anyhow!("dataset not loaded: {dataset}"))?;

        for row in rows {
            if row.get(filter_field).is_some_and(|s| s == filter_value) {
                let value =
                    row.get(desired_field).cloned().ok_or_else(|| {
                        anyhow!(DC_DATA_NO_RESULT_EXCEPTION.to_string())
                    })?;
                return Ok(value);
            }
        }
        Err(anyhow!(DC_DATA_NO_RESULT_EXCEPTION.to_string()))
    }

    /// All matches.
    fn dc_data_filter_by_value(
        &self,
        dataset: &str,
        filter_field: usize,
        filter_value: &str,
        desired_field: usize,
    ) -> Result<Vec<String>> {
        let rows = self
            .data
            .get(dataset)
            .with_context(|| format!("dataset not loaded: {dataset}"))?;
        let mut out = Vec::new();
        for row in rows.iter().take(rows.len()) {
            if row.get(filter_field).is_some_and(|s| s == filter_value) {
                if let Some(v) = row.get(desired_field) {
                    out.push(v.clone());
                }
            }
        }
        Ok(out)
    }

    fn dc_data_filter_by_value_greater(
        &self,
        dataset: &str,
        filter_field: usize,
        filter_value: i32,
        desired_field: usize,
    ) -> Result<Vec<String>> {
        let rows = self
            .data
            .get(dataset)
            .with_context(|| format!("dataset not loaded: {dataset}"))?;
        let mut out = Vec::new();
        for row in rows.iter().take(rows.len()) {
            if let Some(cell) = row.get(filter_field) {
                if let Ok(v) = cell.parse::<i32>() {
                    if v > filter_value {
                        if let Some(d) = row.get(desired_field) {
                            out.push(d.clone());
                        }
                    }
                }
            }
        }
        Ok(out)
    }
}

// Lazy static instance
static EITE_DATA: LazyLock<Result<EiteData>> = LazyLock::new(EiteData::new);

fn get_eite_data() -> Result<&'static EiteData> {
    EITE_DATA
        .as_ref()
        .map_err(|e| anyhow!("Failed to initialize EITE_DATA: {e:?}"))
}

/// Static list of known Dc datasets (mirrors original JS array).
pub fn list_dc_datasets() -> Vec<&'static str> {
    vec![
        "DcData",
        "formats",
        "mappings/from/ascii",
        "mappings/from/unicode",
        "mappings/to/html",
        "mappings/to/lang_en",
        "mappings/to/unicode",
    ]
}

pub fn json() -> Result<String> {
    Ok(get_eite_data()?.json())
}

/// Returns true if the provided dataset name is one of the known Dc datasets.
pub fn is_dc_dataset(name: &str) -> bool {
    list_dc_datasets().iter().any(|s| s == &name)
}

pub fn dc_dataset_length(dataset: &str) -> Result<usize> {
    get_eite_data()?.dc_dataset_length(dataset)
}

pub fn dc_data_get_column(dataset: &str, col_num: usize) -> Result<Vec<String>> {
    get_eite_data()?.dc_data_get_column(dataset, col_num)
}

pub fn dc_data_lookup_by_id(
    dataset: &str,
    row_num: usize,
    field_num: usize,
) -> Result<String> {
    get_eite_data()?.dc_data_lookup_by_id(dataset, row_num, field_num)
}

pub fn dc_data_lookup_by_value(
    dataset: &str,
    filter_field: usize,
    filter_value: &str,
    desired_field: usize,
) -> Result<String> {
    get_eite_data()?.dc_data_lookup_by_value(
        dataset,
        filter_field,
        filter_value,
        desired_field,
    )
}

pub fn dc_data_lookup_by_dc_in_col_0(
    dataset: &str,
    dc: u32,
    desired_field: usize,
) -> Result<String> {
    dc_data_lookup_by_value(dataset, 0, &dc.to_string(), desired_field)
}

pub fn dc_data_filter_by_value(
    dataset: &str,
    filter_field: usize,
    filter_value: &str,
    desired_field: usize,
) -> Result<Vec<String>> {
    get_eite_data()?.dc_data_filter_by_value(
        dataset,
        filter_field,
        filter_value,
        desired_field,
    )
}

pub fn dc_data_filter_by_value_greater(
    dataset: &str,
    filter_field: usize,
    filter_value: i32,
    desired_field: usize,
) -> Result<Vec<String>> {
    get_eite_data()?.dc_data_filter_by_value_greater(
        dataset,
        filter_field,
        filter_value,
        desired_field,
    )
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use crate::{
        dc::{get_dc_count, maximum_known_dc},
        formats::is_format,
    };

    use super::*;

    #[crate::ctb_test]
    fn test_data_loaded() -> Result<()> {
        assert_eq!(dc_dataset_length("DcData")?, 299);
        assert_eq!(get_dc_count()?, 299);
        assert_eq!(maximum_known_dc()?, 298);
        assert!(is_format("unicode"));
        assert!(is_format("utf8"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_list_dc_datasets_contains_expected() {
        let list = list_dc_datasets();
        assert!(list.contains(&"DcData"));
        assert!(list.contains(&"mappings/to/html"));
        assert!(!list.contains(&"nonexistent_dataset_xyz"));
    }
}
