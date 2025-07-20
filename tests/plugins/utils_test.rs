//! Tests for plugin utilities
//!
//! Extracted from src/plugins/examples/utils.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use xreal_virtual_desktop::plugins::examples::utils::*;

#[test]
fn test_quad_vertex_size() {
    // Ensure vertex struct is properly packed
    assert_eq!(std::mem::size_of::<QuadVertex>(), 20); // 3*4 + 2*4 = 20 bytes
}

#[test]
fn test_quad_geometry() {
    let (vertices, indices) = create_quad_vertices();
    assert_eq!(vertices.len(), 4);
    assert_eq!(indices.len(), 6);

    // Verify triangle winding
    assert_eq!(indices[0..3], [0, 1, 2]);
    assert_eq!(indices[3..6], [2, 3, 0]);
}

#[test]
fn test_constants() {
    assert_eq!(QUAD_VERTICES.len(), 4);
    assert_eq!(QUAD_INDICES.len(), 6);
    assert!(!QUAD_SHADER.is_empty());
    assert!(!COLORED_QUAD_SHADER.is_empty());
}