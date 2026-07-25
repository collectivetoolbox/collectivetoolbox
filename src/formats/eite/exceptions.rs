use crate::utilities::*;

use anyhow::Result;

use crate::util::array::str_print_arr;

// ---------------
// Exceptions / sentinel helpers
// ---------------

pub const DC_DATA_NO_RESULT_EXCEPTION: &str =
    "89315802-d53d-4d11-ba5d-bf505e8ed454";
pub const BYTE_ARRAY_FROM_BASENB_UTF8_INVALID_INPUT_EXCEPTION: &str =
    "51 98 218 163 23 5 64 236 154 151 89 208 82 253 64 55 ";

fn _excep_str(s: String) -> bool {
    s == DC_DATA_NO_RESULT_EXCEPTION
        || s == BYTE_ARRAY_FROM_BASENB_UTF8_INVALID_INPUT_EXCEPTION
}
pub fn excep(s: &Result<String>) -> bool {
    match s {
        Ok(_) => _excep_str(s.as_ref().unwrap().clone()),
        Err(e) => {
            let msg = e.to_string();
            if _excep_str(msg.clone()) {
                true
            } else {
                panic!("Unexpected error: {msg}");
            }
        }
    }
}
pub fn not_excep(s: &Result<String>) -> bool {
    !excep(s)
}
pub fn excep_arr<T: ToString>(arr: &[T]) -> bool {
    let printed = str_print_arr(arr);
    excep(&Ok(printed))
}
pub fn not_excep_arr<T: ToString>(arr: &[T]) -> bool {
    !excep_arr(arr)
}

/// Returns true if the result is empty or an exception marker value.
/// Returns false if the result is not empty, or if the result is an Err other
/// than an exception marker value.
pub fn exc_or_empty(s: &Result<String>) -> bool {
    (s.is_ok() && s.as_ref().unwrap().is_empty()) || excep(s)
}
pub fn not_exc_or_empty(s: &Result<String>) -> bool {
    !exc_or_empty(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::anyhow;

    #[crate::ctb_test]
    fn test_exceptions() {
        assert!(excep(&Ok(DC_DATA_NO_RESULT_EXCEPTION.to_string())));
        assert!(excep(&Err(anyhow!(DC_DATA_NO_RESULT_EXCEPTION))));
        assert!(not_excep(&Ok("normal".to_string())));
    }
}
