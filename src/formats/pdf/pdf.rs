//! PDF conversion helpers.
//! FIXME: TODO!

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::Result;
#[cfg(test)]
use include_dir::{Dir, include_dir};
// use unpdf::render::{RenderOptions, JsonFormat, to_text, to_json, to_markdown};

#[cfg(test)]
static PDF_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

#[cfg(test)]
pub(crate) fn get_pdf_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&PDF_DATA_DIR, key)
}

/// Extract raw text from a PDF byte array.
pub fn pdf2txt(_input: &[u8]) -> Result<String> {
    Ok(String::new())
    // let doc = unpdf::parse_bytes(input)?;
    // let options = RenderOptions::default();
    // let txt = to_text(&doc, &options)?;
    // Ok(txt)
}

/// Convert PDF byte array structure into JSON format.
pub fn pdf2json(_input: &[u8]) -> Result<String> {
    Ok(String::new())
    // let doc = unpdf::parse_bytes(input)?;
    // let json = to_json(&doc, JsonFormat::Pretty)?;
    // Ok(json)
}

/// Convert PDF byte array content to Markdown format.
pub fn pdf2md(_input: &[u8]) -> Result<String> {
    Ok(String::new())
    // let doc = unpdf::parse_bytes(input)?;
    // let options = RenderOptions::default();
    // let md = to_markdown(&doc, &options)?;
    // Ok(md)
}

#[cfg(test)]
#[expect(
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

    // #[crate::ctb_test]
    // fn test_pdf2txt_invalid() {
    //     assert!(pdf2txt(b"invalid pdf").is_err());
    // }

    // #[crate::ctb_test]
    // fn test_pdf2json_invalid() {
    //     assert!(pdf2json(b"invalid pdf").is_err());
    // }

    // #[crate::ctb_test]
    // fn test_pdf2md_invalid() {
    //     assert!(pdf2md(b"invalid pdf").is_err());
    // }

    // #[crate::ctb_test]
    // fn test_pdf2txt_valid() {
    //     let pdf_data = get_pdf_data("fixtures/sample.pdf").unwrap();
    //     let txt = pdf2txt(&pdf_data).unwrap();

    //     // unpdf extracts raw text with some null characters and spacing due to font encoding.
    //     // We clean up nulls, spaces, and form-feed characters to compare the semantic text content.
    //     let txt_clean = txt.replace('\0', "").replace(' ', "").replace("\r", "");

    //     let expected_txt_bytes = get_pdf_data("fixtures/sample.txt").unwrap();
    //     let expected_txt = String::from_utf8(expected_txt_bytes).unwrap();
    //     let expected_clean = expected_txt.replace(' ', "").replace('\u{c}', "").replace("\r", "");

    //     assert_eq!(txt_clean.trim(), expected_clean.trim());
    // }

    // #[crate::ctb_test]
    // fn test_pdf2json_valid() {
    //     let pdf_data = get_pdf_data("fixtures/sample.pdf").unwrap();
    //     let json = pdf2json(&pdf_data).unwrap();
    //     let json_clean = json.replace("\\u0000", "").replace(' ', "");
    //     assert!(json_clean.contains("6/29/26"));
    // }

    // #[crate::ctb_test]
    // fn test_pdf2md_valid() {
    //     let pdf_data = get_pdf_data("fixtures/sample.pdf").unwrap();
    //     let md = pdf2md(&pdf_data).unwrap();
    //     let md_clean = md.replace('\0', "").replace(' ', "");
    //     assert!(md_clean.contains("6/29/26"));
    // }

    // TODO: unpdf is not currently capable of handling sample-invalid.pdf (extracts empty string instead of spacing/form-feed).
    // #[crate::ctb_test]
    // fn test_pdf2txt_sample_invalid() {
    //     let pdf_data = get_pdf_data("fixtures/sample-invalid.pdf").unwrap();
    //     let txt = pdf2txt(&pdf_data).unwrap();
    //     let expected_txt_bytes = get_pdf_data("fixtures/sample-invalid.txt").unwrap();
    //     let expected_txt = String::from_utf8(expected_txt_bytes).unwrap();
    //     assert_eq!(txt, expected_txt);
    // }
}
