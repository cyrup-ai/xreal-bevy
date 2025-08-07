//! Fast data structures for plugin system (stub)

#![allow(dead_code)]

use crate::plugins::{PluginAuthor, PluginDescription, PluginId, PluginName, PluginVersion};

// Stub implementations
pub fn create_plugin_id(id: &str) -> PluginId {
    PluginId::from(id)
}

pub fn create_plugin_name(name: &str) -> PluginName {
    PluginName::from(name)
}

pub fn create_plugin_version(version: &str) -> PluginVersion {
    PluginVersion::from(version)
}

pub fn create_plugin_description(desc: &str) -> PluginDescription {
    PluginDescription::from(desc)
}

pub fn create_plugin_author(author: &str) -> PluginAuthor {
    PluginAuthor::from(author)
}
