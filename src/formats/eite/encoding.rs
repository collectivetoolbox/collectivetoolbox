use crate::{
    dc::data::dc_data_filter_by_value, formats::is_supported_output_format,
};

pub mod ascii;
pub mod base;
pub mod basenb;
pub mod pack32;
pub mod unicode;
pub mod utf8;

pub fn list_char_encodings() -> Vec<String> {
    dc_data_filter_by_value("formats", 6, "encoding", 1)
}

pub fn is_supported_char_encoding(fmt: &str) -> bool {
    list_char_encodings()
        .iter()
        .any(|f| f == fmt && is_supported_output_format(fmt))
}
