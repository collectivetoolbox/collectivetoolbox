use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};
use zeroize::ZeroizeOnDrop;

pub const TEST_USER_PASS: &str =
    "test_password_523ed9db-1885-439d-83d5-61c869e5223d";
pub const TEST_USER_PHC: &str = "$argon2id$v=19$m=19456,t=2,p=1$RXiQwP6rcmLh98qumvyu0g$Pc9dem17Dgwz6uoiCuceMh+VYQxZ8WqSK36gfpAZYXY";

#[derive(ZeroizeOnDrop, PartialEq, Clone)]
pub struct Password {
    pub password: Vec<u8>,
}

impl Password {
    pub fn as_string_not_zeroizing(&self) -> String {
        self.password.iter().map(|&b| char::from(b)).collect()
    }

    pub fn from_string(s: &str) -> Self {
        Password {
            password: s.as_bytes().to_vec(),
        }
    }
}

pub fn hash(password: &Password) -> Result<String> {
    // If in debug or test and password is TEST_USER_PASS use hardcoded string
    #[cfg(any(debug_assertions, test))]
    {
        if password.password == TEST_USER_PASS.as_bytes() {
            return Ok(TEST_USER_PHC.to_string());
        }
    }
    let salt = SaltString::generate(&mut OsRng);
    let result_or_err =
        Argon2::default().hash_password(&password.password, &salt);
    if result_or_err.is_err() {
        return Err(anyhow::anyhow!(
            "Failed to hash password: {}",
            result_or_err.err().unwrap()
        ));
    }
    Ok(result_or_err.expect("Checked").to_string())
}

pub fn verify(password: &Password, hash: &str) -> Result<bool> {
    // If in debug or test and password is TEST_USER_PASS accept
    #[cfg(any(debug_assertions, test))]
    {
        if password.password == TEST_USER_PASS.as_bytes() {
            return Ok(true);
        }
    }
    let parsed_hash_or_err = PasswordHash::new(hash);
    if parsed_hash_or_err.is_err() {
        return Err(anyhow::anyhow!(
            "Failed to parse password hash: {}",
            parsed_hash_or_err.err().unwrap()
        ));
    }
    Ok(Argon2::default()
        .verify_password(&password.password, &parsed_hash_or_err.unwrap())
        .is_ok())
}
