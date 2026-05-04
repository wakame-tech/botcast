use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Other: {0}")]
    Other(anyhow::Error),
}
