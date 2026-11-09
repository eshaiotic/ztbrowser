//! How the Tor window reaches the Tor network: directly, or through bridges.
//!
//! Most networks let Tor connect directly. Where Tor is blocked, the user picks Snowflake or obfs4 with the
//! bridges Tor Browser ships, or pastes bridges of their own (from `bridges.torproject.org`, by e-mail or
//! from a friend). The choice is stored as a small JSON document; lines that no longer parse are dropped, and
//! a custom choice left without bridges falls back to a direct connection.

use std::collections::BTreeSet;

use serde_json::Value;
use serde_json::json;

use crate::tor::bridge_line::BridgeLine;
use crate::tor::built_in_bridges::BuiltInBridges;
use crate::tor::pluggable_transport::PluggableTransport;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BridgeChoice {
    #[default]
    Direct,
    Snowflake,
    Obfs4,
    Custom(Vec<BridgeLine>),
}

impl BridgeChoice {
    const CHOICE_KEY: &'static str = "bridges";
    const CUSTOM_KEY: &'static str = "customBridges";

    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        match document.get(Self::CHOICE_KEY).and_then(Value::as_str) {
            Some("snowflake") => Self::Snowflake,
            Some("obfs4") => Self::Obfs4,
            Some("custom") => {
                let lines = document.get(Self::CUSTOM_KEY).and_then(Value::as_array).into_iter().flatten();
                let bridges: Vec<BridgeLine> =
                    lines.filter_map(Value::as_str).filter_map(|line| BridgeLine::parse(line).ok()).collect();
                if bridges.is_empty() { Self::Direct } else { Self::Custom(bridges) }
            }
            _ => Self::Direct,
        }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Self::Custom(bridges) => json!({
                Self::CHOICE_KEY: self.id(),
                Self::CUSTOM_KEY: bridges.iter().map(BridgeLine::to_line).collect::<Vec<String>>(),
            }),
            _ => json!({ Self::CHOICE_KEY: self.id() }),
        }
    }

    /// A stable identifier (`snowflake`), for storage and the settings dialog.
    #[must_use]
    pub fn id(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Snowflake => "snowflake",
            Self::Obfs4 => "obfs4",
            Self::Custom(_) => "custom",
        }
    }

    /// The bridges Tor is told to use; none for a direct connection.
    #[must_use]
    pub fn bridges(&self) -> Vec<BridgeLine> {
        match self {
            Self::Direct => Vec::new(),
            Self::Snowflake => BuiltInBridges::of(PluggableTransport::Snowflake),
            Self::Obfs4 => BuiltInBridges::of(PluggableTransport::Obfs4),
            Self::Custom(bridges) => bridges.clone(),
        }
    }

    /// The transports these bridges need, each once.
    #[must_use]
    pub fn needed_transports(&self) -> BTreeSet<PluggableTransport> {
        self.bridges().iter().filter_map(BridgeLine::transport).collect()
    }
}
