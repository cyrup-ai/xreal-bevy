//! Plugin surface management (stub)

#![allow(dead_code)]

use anyhow::Result;
use bevy::prelude::*;

#[derive(Resource)]
pub struct SurfaceManager;

impl SurfaceManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn get_visible_surfaces(&self) -> Vec<String> {
        vec![]
    }

    pub fn get_total_memory_usage(&self) -> u64 {
        0
    }

    pub fn update_surface_transform(&mut self, _id: &str, _pos: Vec3, _visible: bool) -> Result<()> {
        Ok(())
    }

    pub fn resize_surface(&mut self, _id: &str, _size: (u32, u32)) -> Result<()> {
        Ok(())
    }
}

#[derive(Resource, Default)]
pub struct PluginWindowManager;

impl PluginWindowManager {
    pub fn get_focused_plugin(&self) -> Option<String> {
        None
    }

    pub fn focus_plugin(&mut self, _id: String) {}
    pub fn unfocus_plugin(&mut self, _id: &str) {}
}

pub fn surface_management_system() {}
pub fn plugin_render_system() {}
pub fn update_plugin_surface_positions() {}
pub fn plugin_window_focus_system() {}