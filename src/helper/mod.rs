use std::env;

pub fn get_env_var(key: &str) -> Result<String, String> {
    match env::var(key) {
        Ok(val) => Ok(val),
        Err(_) => Err(format!("{} must be set", key)),
    }
}
