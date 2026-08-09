use crate::utilities::*;
use crate::dc::data::dc_data_filter_by_value;
use crate::formats::is_supported_output_format;

pub fn list_terminal_types() -> Result<Vec<String>> {
    dc_data_filter_by_value("formats", 6, "terminal", 1)
}

pub fn is_supported_terminal_type(fmt: &str) -> bool {
    let Ok(terminals) = list_terminal_types() else {
        return false;
    };
    terminals
        .iter()
        .any(|f| f == fmt && is_supported_output_format(fmt))
}
