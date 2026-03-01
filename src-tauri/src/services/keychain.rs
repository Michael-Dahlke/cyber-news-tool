use keyring::Entry;

const SERVICE_NAME: &str = "cyber-news-reader";
const ACCOUNT_NAME: &str = "llm-api-key";

/// Store the LLM API key in the OS keychain.
pub fn set_api_key(api_key: &str) -> Result<(), String> {
    if api_key.is_empty() {
        return Err("EMPTY_KEY".to_string());
    }
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|e| format!("KEYCHAIN_ERROR: {e}"))?;
    entry
        .set_password(api_key)
        .map_err(|e| format!("KEYCHAIN_ERROR: {e}"))
}

/// Retrieve the LLM API key from the OS keychain.
pub fn get_api_key() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|e| format!("KEYCHAIN_ERROR: {e}"))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("KEYCHAIN_ERROR: {e}")),
    }
}

/// Check whether an API key exists in the OS keychain without returning it.
pub fn has_api_key() -> bool {
    matches!(get_api_key(), Ok(Some(_)))
}

/// Remove the LLM API key from the OS keychain.
pub fn clear_api_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|e| format!("KEYCHAIN_ERROR: {e}"))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already gone — not an error
        Err(e) => Err(format!("KEYCHAIN_ERROR: {e}")),
    }
}
