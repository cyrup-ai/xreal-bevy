//! Fast plugin registry (stub)

#![allow(dead_code)]

use anyhow::Result;
use bevy::prelude::*;
use crate::state::schema::plugins::*;

#[derive(Resource)]
pub struct FastPluginRegistry {
    config: PluginSystemConfig,
}

impl FastPluginRegistry {
    pub fn new(config: PluginSystemConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn get_plugin(&self, _id: &str) -> Option<PluginEntry> {
        None
    }

    pub fn list_active_plugins(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }

    pub fn update_plugin_state(&self, _id: &str, _state: u64) -> Result<()> {
        Ok(())
    }

    pub fn record_performance(&self, _id: &str, _time_us: u32) -> Result<()> {
        Ok(())
    }
}

pub struct PluginEntry {
    pub state: AtomicPluginState,
    pub metadata: PluginMetadata,
}

impl PluginEntry {
    pub fn get_state(&self) -> u64 {
        0
    }
}

pub fn fast_plugin_event_system() {
    // Stub system
}

#[derive(Debug, Clone)]
pub enum FastPluginEvent {
    None,
    PluginLoaded { plugin_id: SmallString },
    PluginInitialized { plugin_id: SmallString },
    PluginStarted { plugin_id: SmallString },
    PluginPaused { plugin_id: SmallString },
    PluginError { plugin_id: SmallString },
    PluginUnloaded { plugin_id: SmallString },
    PerformanceViolation { plugin_id: SmallString },
}