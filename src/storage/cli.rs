#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;


#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;


    #[crate::ctb_test("tokio")]
    async fn test_adduser_command() -> Result<()> {
        assert!(crate::routing::is_lightweight_command("adduser"));

        let username = format!("cli_user_{}", function_name!());
        ctb_storage::user::User::delete_by_name(&username).ok();

        // Ensure allow_local_account_creation is true for this test
        let mut settings =
            ctb_utilities::pc_settings::PcSettings::load().unwrap_or_default();
        settings.allow_local_account_creation =
            ctb_utilities::json::maybe_value::MaybeValue::Value(true);
        settings.save()?;

        let _cmd = Command::AddUser {
            username: username.clone(),
            password_stdin: true,
        };

        // Note: Password comes from stdin or add_non_admin_user directly.
        // Let's test calling add_non_admin_user
        let password = ctb_utilities::password::Password::from_string(
            ctb_utilities::password::TEST_USER_PASS,
        );
        let user = ctb_storage::user::add_non_admin_user(&username, &password)?;
        assert_eq!(user.name(), username);
        assert!(!user.is_admin());

        user.delete()?;
        Ok(())
    }

}