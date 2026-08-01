#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

#[allow(clippy::uninlined_format_args, reason = "Much more readable in this case")]
pub fn ed25519_base64_to_pem(ed25519: &str) -> String {
    // From my brief reading, it seems that this prefix is ASN.1 notation indicating that it is an Ed25519 key, encoded using DER, and then encoded using base64. FIXME: It would probably be nice to generate this some way that is clear what it actually is, rather than an unreadable string.
    let prefix = "MCowBQYDK2VwAyEA";
    format!(
        "-----BEGIN PUBLIC KEY-----\n{}{}\n-----END PUBLIC KEY-----\n",
        prefix, ed25519
    )
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {}
