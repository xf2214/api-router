#[cfg(not(test))]
use secrecy::ExposeSecret;
use secrecy::SecretString;

use crate::error::AppError;

#[allow(dead_code)]
const SERVICE_NAME: &str = "api-router";

#[cfg(test)]
static MEM_STORE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, String>>> =
    std::sync::OnceLock::new();
#[cfg(test)]
fn mem_store() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>> {
    MEM_STORE.get_or_init(Default::default)
}

/// Store an API key for a provider.
#[allow(clippy::needless_return)]
pub fn set_key(provider_id: &str, api_key: &str) -> Result<(), AppError> {
    #[cfg(test)]
    {
        mem_store()
            .lock()
            .unwrap()
            .insert(provider_id.to_string(), api_key.to_string());
        return Ok(());
    }
    #[cfg(not(test))]
    {
        let entry = keyring::Entry::new(SERVICE_NAME, provider_id)?;
        entry.set_password(api_key)?;
        Ok(())
    }
}

/// Retrieve an API key for a provider.
#[allow(dead_code)]
#[allow(clippy::needless_return)]
pub fn get_key(provider_id: &str) -> Result<SecretString, AppError> {
    #[cfg(test)]
    {
        let store = mem_store().lock().unwrap();
        if let Some(v) = store.get(provider_id) {
            return Ok(SecretString::from(v.clone()));
        } else {
            return Err(AppError::Keyring("test store miss".into()));
        }
    }
    #[cfg(not(test))]
    {
        let entry = keyring::Entry::new(SERVICE_NAME, provider_id)?;
        let password = entry.get_password()?;
        Ok(SecretString::from(password))
    }
}

/// Clear an API key for a provider by setting it to an empty string.
#[allow(clippy::needless_return)]
pub fn clear_key(provider_id: &str) -> Result<(), AppError> {
    #[cfg(test)]
    {
        mem_store().lock().unwrap().remove(provider_id);
        return Ok(());
    }
    #[cfg(not(test))]
    {
        set_key(provider_id, "")
    }
}

/// Retrieve the raw API key string. Avoid logging this value.
#[allow(clippy::needless_return)]
pub fn get_key_string(provider_id: &str) -> Result<String, AppError> {
    #[cfg(test)]
    {
        let store = mem_store().lock().unwrap();
        if let Some(v) = store.get(provider_id) {
            return Ok(v.clone());
        } else {
            return Err(AppError::Keyring("test store miss".into()));
        }
    }
    #[cfg(not(test))]
    {
        let secret = get_key(provider_id)?;
        Ok(secret.expose_secret().to_string())
    }
}
