use turborepo_errors::ParseDiagnostic;

use crate::SUPPORTED_VERSIONS;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unable to parse JSON: {0}")]
    JsonParse(String),
    #[error(
        "Unsupported microfrontends configuration version: {0}. Supported versions: \
         {SUPPORTED_VERSIONS:?}"
    )]
    UnsupportedVersion(String),
    #[error("Configuration references config located in package {reference}.")]
    ChildConfig { reference: String },
    #[error("`{path}`: Cannot parse config with version '{actual}' as version '{expected}'.")]
    InvalidVersion {
        expected: &'static str,
        actual: String,
        path: String,
    },
}

impl Error {
    /// Constructs an error message from multiple biome diagnostic errors
    pub fn biome_error(errors: Vec<biome_diagnostics::Error>) -> Self {
        let error_messages = errors
            .into_iter()
            .map(|err| ParseDiagnostic::from(err.as_ref()).to_string())
            .collect::<Vec<_>>();
        Self::JsonParse(error_messages.join("\n"))
    }
}
