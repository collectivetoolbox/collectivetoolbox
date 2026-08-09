//! Cryptographic blind signature helpers using VOPRF (RFC 9497).
//! Uses Ristretto255 and the `voprf` crate.
//! I haven't checked this for correctness!

#[expect(unused_imports, reason = "Standard workspace prelude")]
use crate::utilities::*;
use anyhow::{Result, anyhow};

pub use voprf::{
    BlindedElement, EvaluationElement, OprfClient, OprfServer, Ristretto255,
};

/// The size of a serialized server private key (Ristretto255 scalar).
pub const SERVER_KEY_SIZE: usize = 32;

/// Generate a new random OPRF server instance.
pub fn generate_server_key() -> Result<Vec<u8>> {
    let mut rng = rand_08::thread_rng();
    let server = OprfServer::<Ristretto255>::new(&mut rng)
        .map_err(|e| anyhow!("Failed to generate server key: {e:?}"))?;
    Ok(server.serialize().to_vec())
}

/// Helper to serialize a `BlindedElement` to bytes.
pub fn serialize_blinded(blinded: &BlindedElement<Ristretto255>) -> Vec<u8> {
    blinded.serialize().to_vec()
}

/// Helper to deserialize a `BlindedElement` from bytes.
pub fn deserialize_blinded(
    bytes: &[u8],
) -> Result<BlindedElement<Ristretto255>> {
    BlindedElement::<Ristretto255>::deserialize(bytes)
        .map_err(|e| anyhow!("Failed to deserialize BlindedElement: {e:?}"))
}

/// Helper to serialize an `EvaluationElement` to bytes.
pub fn serialize_evaluation(
    evaluation: &EvaluationElement<Ristretto255>,
) -> Vec<u8> {
    evaluation.serialize().to_vec()
}

/// Helper to deserialize an `EvaluationElement` from bytes.
pub fn deserialize_evaluation(
    bytes: &[u8],
) -> Result<EvaluationElement<Ristretto255>> {
    EvaluationElement::<Ristretto255>::deserialize(bytes)
        .map_err(|e| anyhow!("Failed to deserialize EvaluationElement: {e:?}"))
}

/// Client step 1: Blind the serial input.
/// Returns (`blinded_element_bytes`, `client_state_bytes`).
pub fn client_blind(serial: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut rng = rand_08::thread_rng();
    let result = OprfClient::<Ristretto255>::blind(serial, &mut rng)
        .map_err(|e| anyhow!("Failed to blind input: {e:?}"))?;

    let blinded_bytes = result.message.serialize().to_vec();
    let client_bytes = result.state.serialize().to_vec();
    Ok((blinded_bytes, client_bytes))
}

/// Server step 2: Sign/evaluate the blinded element bytes using the server private key bytes.
/// Returns serialized `evaluation_element_bytes`.
pub fn server_evaluate(
    server_key: &[u8],
    blinded_element_bytes: &[u8],
) -> Result<Vec<u8>> {
    let server = OprfServer::<Ristretto255>::deserialize(server_key)
        .map_err(|e| anyhow!("Failed to deserialize server key: {e:?}"))?;
    let blinded = deserialize_blinded(blinded_element_bytes)?;

    let evaluation = server.blind_evaluate(&blinded);
    Ok(evaluation.serialize().to_vec())
}

/// Client step 3: Unblind/finalize the server evaluation bytes using the original serial and client state.
/// Returns the unblinded token/tag bytes.
pub fn client_finalize(
    serial: &[u8],
    client_state_bytes: &[u8],
    evaluation_element_bytes: &[u8],
) -> Result<Vec<u8>> {
    let client = OprfClient::<Ristretto255>::deserialize(client_state_bytes)
        .map_err(|e| anyhow!("Failed to deserialize client state: {e:?}"))?;
    let evaluation = deserialize_evaluation(evaluation_element_bytes)?;

    let output = client
        .finalize(serial, &evaluation)
        .map_err(|e| anyhow!("Failed to finalize/unblind evaluation: {e:?}"))?;
    Ok(output.to_vec())
}

/// Server spending verification: Check if unblinded token tag matches the server's evaluation of the serial.
pub fn server_verify(
    server_key: &[u8],
    serial: &[u8],
    token_tag: &[u8],
) -> Result<bool> {
    let server = OprfServer::<Ristretto255>::deserialize(server_key)
        .map_err(|e| anyhow!("Failed to deserialize server key: {e:?}"))?;

    let expected = server
        .evaluate(serial)
        .map_err(|e| anyhow!("Failed server evaluation of serial: {e:?}"))?;
    Ok(expected.to_vec() == token_tag)
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
    use super::{
        client_blind, client_finalize, generate_server_key, server_evaluate,
        server_verify,
    };
    use anyhow::Result;

    #[crate::ctb_test]
    fn test_voprf_flow() -> Result<()> {
        let server_key = generate_server_key()?;
        let serial = b"my-unique-token-serial-123456789";

        // Client blinds
        let (blinded, client_state) = client_blind(serial)?;

        // Server evaluates
        let evaluation = server_evaluate(&server_key, &blinded)?;

        // Client unblinds
        let token_tag = client_finalize(serial, &client_state, &evaluation)?;

        // Server verifies spending
        let is_valid = server_verify(&server_key, serial, &token_tag)?;
        assert!(is_valid, "OPRF verification failed");

        // Verify that a different serial fails verification
        let is_valid_diff =
            server_verify(&server_key, b"different-serial", &token_tag)?;
        assert!(!is_valid_diff, "OPRF verified a wrong serial");

        Ok(())
    }
}
