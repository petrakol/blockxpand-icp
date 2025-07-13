#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    Network(String),
    InvalidConfig(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<ic_agent::AgentError> for FetchError {
    fn from(e: ic_agent::AgentError) -> Self {
        FetchError::Network(e.to_string())
    }
}
