use secrecy::{ExposeSecret, SecretString};

use crate::error::AppError;

const SERVICE_NAME: &str = "api-router";

/// Store an API key for a provider.
pub fn set_key(provider_id: &str, api_key: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)?;
    entry.set_password(api_key)?;
    Ok(())
}

/// Retrieve an API key for a provider.
pub fn get_key(provider_id: &str) -> Result<SecretString, AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)?;
    let password = entry.get_password()?;
    Ok(SecretString::from(password))
}

/// Clear an API key for a provider by setting it to an empty string.
pub fn clear_key(provider_id: &str) -> Result<(), AppError> {
    set_key(provider_id, "")
}

/// Retrieve the raw API key string. Avoid logging this value.
pub fn get_key_string(provider_id: &str) -> Result<String, AppError> {
    let secret = get_key(provider_id)?;
    Ok(secret.expose_secret().to_string())
}
