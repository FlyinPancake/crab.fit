use figment2::providers::Env;
use garde::Validate;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use thiserror::Error;

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdaptorConfig {
    #[default]
    Memory,
    Sql,
    Datastore, // GCP config will be loaded from environment variables
}

#[derive(Debug, Deserialize, Validate, Clone)]
#[garde(context(AdaptorConfig as adaptor))]
#[serde_inline_default]
pub struct Config {
    #[serde_inline_default(AdaptorConfig::Memory)]
    #[garde(skip)]
    pub adaptor_kind: AdaptorConfig,
    #[garde(custom(validate_database_url))]
    pub database_url: Option<String>,

    #[garde(skip)]
    #[serde_inline_default("http://localhost:1234".to_string())]
    // URL of the frontend application
    // Used for CORS configuration
    pub frontend_url: String,

    #[garde(skip)]
    pub cron_key: Option<String>,

    #[garde(skip)]
    #[serde_inline_default("0.0.0.0:3000".to_string())]
    pub http_listen_addr: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            adaptor_kind: Default::default(),
            database_url: None,
            frontend_url: "http://localhost:1234".to_string(),
            cron_key: None,
            http_listen_addr: "0.0.0.0:3000".to_string(),
        }
    }
}

fn validate_database_url(url: &Option<String>, adaptor: &AdaptorConfig) -> garde::Result {
    if matches!(adaptor, AdaptorConfig::Sql) && url.is_none() {
        return Err(garde::Error::new(
            "database_url is required for SQL adaptor",
        ));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("figment error: {0}")]
    Figment(#[from] figment2::Error),
    #[error("validation error: {0}")]
    Validation(#[from] garde::Report),
}

impl Config {
    #[expect(clippy::result_large_err)]
    pub fn load() -> Result<Self, ConfigError> {
        let c: Config = figment2::Figment::new()
            .merge(Env::prefixed("CRABFIT_"))
            // Compat for legacy environment variables
            .merge(Env::raw().only(&["DATABASE_URL", "CRON_KEY", "FRONTEND_URL"]))
            .extract()?;
        c.validate_with(&c.adaptor_kind)?;

        Ok(c)
    }
}
