//! ZTBrowser's private Tor for the Tor window: its configuration, its control protocol, its bridges.
//!
//! The Tor window's process starts its own `tor` (never the system's Tor service) with a configuration
//! written here, talks to it over Tor's control port, and sends all of the window's traffic to the SOCKS port
//! Tor reports. Nothing here starts processes or opens sockets: the `ztbrowser` application does that, with
//! the values these types produce and check.

pub mod bootstrap_status;
pub mod bridge_choice;
pub mod bridge_line;
pub mod built_in_bridges;
pub mod control_command;
pub mod control_reply;
pub mod keyword_arguments;
pub mod pluggable_transport;
pub mod tor_configuration;
pub mod tor_endpoint;
pub mod tor_programs;
