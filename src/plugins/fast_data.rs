//! Fast data structures for plugin system (stub)

#![allow(dead_code)]

pub use crate::state::schema::plugins::*;

// Stub implementations
pub fn create_plugin_id(id: &str) -> PluginId {
    PluginId::new(id.to_string())
}

pub fn create_plugin_name(name: &str) -> PluginName {
    PluginName::new(name.to_string())
}

pub fn create_plugin_version(version: &str) -> PluginVersion {
    PluginVersion::new(version.to_string())
}

pub fn create_plugin_description(desc: &str) -> PluginDescription {
    PluginDescription::new(desc.to_string())
}

pub fn create_plugin_author(author: &str) -> PluginAuthor {
    PluginAuthor::new(author.to_string())
}