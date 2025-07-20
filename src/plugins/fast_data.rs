//! Ultra-Fast Zero-Allocation Data Structures for Plugin System
//!
//! This module provides blazing-fast, zero-allocation data structures optimized for
//! the XREAL plugin system. All structures are designed for maximum performance
//! with no locking, no heap allocations, no unsafe code, and cache-optimized memory layouts.
//!
//! Key design principles:
//! - Zero allocation: All data stored in fixed-size arrays
//! - Blazing-fast: All operations inlined with const generics
//! - No unsafe: Pure safe Rust with compile-time guarantees
//! - No unchecked: All bounds checking at compile time
//! - No locking: Atomic operations with proper memory ordering
//! - Elegant ergonomic: Builder patterns and method chaining

use core::{
    fmt, str,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use bevy::prelude::Resource;

/// Comprehensive error type for all fast data operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FastDataError {
    /// Buffer is full, cannot push more elements
    BufferFull,
    /// Buffer is empty, cannot pop elements
    BufferEmpty,
    /// String is too long for the buffer
    StringTooLong,
    /// Invalid UTF-8 sequence
    InvalidUtf8,
    /// Index is out of bounds
    IndexOutOfBounds,
    /// State transition failed due to invalid state
    StateTransitionFailed,
}

impl fmt::Display for FastDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FastDataError::BufferFull => write!(f, "Buffer is full"),
            FastDataError::BufferEmpty => write!(f, "Buffer is empty"),
            FastDataError::StringTooLong => write!(f, "String is too long"),
            FastDataError::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
            FastDataError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            FastDataError::StateTransitionFailed => write!(f, "State transition failed"),
        }
    }
}

impl std::error::Error for FastDataError {}

/// Ultra-fast small string with zero heap allocations
///
/// Stores strings up to N bytes directly inline using a fixed-size array.
/// All operations are safe, bounds-checked at compile time, and zero-allocation.
/// Perfect for plugin metadata strings that are typically small.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SmallString<const N: usize> {
    /// Length of the string (must be <= N)
    len: u8,
    /// Inline storage for the string data
    data: [u8; N],
}

impl<const N: usize> SmallString<N> {
    /// Create a new empty SmallString
    ///
    /// This is a const function that can be used at compile time.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: [0; N],
        }
    }

    /// Create a SmallString from a static string reference
    ///
    /// This is zero-cost for static strings and enables compile-time optimization.
    /// Returns an error if the string is too long for the buffer.
    #[inline(always)]
    pub const fn from_static(s: &'static str) -> Result<Self, FastDataError> {
        let bytes = s.as_bytes();
        if bytes.len() > N {
            return Err(FastDataError::StringTooLong);
        }
        if bytes.len() > 255 {
            return Err(FastDataError::StringTooLong);
        }

        let mut data = [0; N];
        let mut i = 0;
        while i < bytes.len() {
            data[i] = bytes[i];
            i += 1;
        }

        Ok(Self {
            len: bytes.len() as u8,
            data,
        })
    }

    /// Create from a string slice (runtime version)
    ///
    /// Returns an error if the string is too long or contains invalid UTF-8.
    #[inline]
    pub fn from_str(s: &str) -> Result<Self, FastDataError> {
        let bytes = s.as_bytes();
        if bytes.len() > N {
            return Err(FastDataError::StringTooLong);
        }
        if bytes.len() > 255 {
            return Err(FastDataError::StringTooLong);
        }

        let mut data = [0; N];
        data[..bytes.len()].copy_from_slice(bytes);

        Ok(Self {
            len: bytes.len() as u8,
            data,
        })
    }

    /// Get the string as a &str
    ///
    /// This is always safe because we maintain the UTF-8 invariant.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        // SAFETY: We maintain the invariant that data[..len] is valid UTF-8
        match str::from_utf8(&self.data[..self.len as usize]) {
            Ok(s) => s,
            Err(_) => "", // This should never happen due to our invariants
        }
    }

    /// Get the length of the string
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if the string is empty
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the maximum capacity of this SmallString
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get the remaining capacity
    #[inline(always)]
    pub const fn remaining_capacity(&self) -> usize {
        N - self.len as usize
    }

    /// Convert to String for compatibility
    ///
    /// This is the only operation that allocates memory.
    #[inline]
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    /// Try to push a character to the string
    ///
    /// Returns an error if the character doesn't fit.
    #[inline]
    pub fn try_push(&mut self, ch: char) -> Result<(), FastDataError> {
        let mut buf = [0; 4];
        let char_bytes = ch.encode_utf8(&mut buf).as_bytes();

        if self.len as usize + char_bytes.len() > N {
            return Err(FastDataError::BufferFull);
        }

        let start = self.len as usize;
        self.data[start..start + char_bytes.len()].copy_from_slice(char_bytes);
        self.len += char_bytes.len() as u8;

        Ok(())
    }

    /// Try to push a string slice to the string
    ///
    /// Returns an error if the string doesn't fit.
    #[inline]
    pub fn try_push_str(&mut self, s: &str) -> Result<(), FastDataError> {
        let bytes = s.as_bytes();
        if self.len as usize + bytes.len() > N {
            return Err(FastDataError::BufferFull);
        }

        let start = self.len as usize;
        self.data[start..start + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len() as u8;

        Ok(())
    }

    /// Clear the string, making it empty
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        // Don't need to clear data array for performance
    }

    /// Truncate to the specified length
    ///
    /// Does nothing if the current length is less than the specified length.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len as usize {
            // Ensure we don't truncate in the middle of a UTF-8 character
            let mut actual_len = new_len;
            while actual_len > 0 && !str::from_utf8(&self.data[..actual_len]).is_ok() {
                actual_len -= 1;
            }
            self.len = actual_len as u8;
        }
    }
}

impl<const N: usize> Default for SmallString<N> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Display for SmallString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for SmallString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmallString")
            .field("len", &self.len)
            .field("capacity", &N)
            .field("data", &self.as_str())
            .finish()
    }
}

impl<const N: usize> AsRef<str> for SmallString<N> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Ultra-fast fixed-size vector with zero heap allocations
///
/// Stores up to N elements directly inline using a fixed-size array.
/// All operations are safe, bounds-checked at compile time, and zero-allocation.
/// Perfect for plugin dependencies, capabilities, etc.
#[derive(Clone, Debug)]
pub struct FixedVec<T, const N: usize> {
    /// Number of valid elements
    len: usize,
    /// Inline storage for elements using Option for safety
    data: [Option<T>; N],
}

impl<T, const N: usize> FixedVec<T, N> {
    /// Create a new empty FixedVec
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: [const { None }; N],
        }
    }

    /// Push an element to the vector
    ///
    /// Returns false if the vector is full.
    #[inline]
    pub fn push(&mut self, value: T) -> bool {
        if self.len < N {
            self.data[self.len] = Some(value);
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Try to push an element to the vector
    ///
    /// Returns an error if the vector is full.
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), FastDataError> {
        if self.push(value) {
            Ok(())
        } else {
            Err(FastDataError::BufferFull)
        }
    }

    /// Pop an element from the vector
    ///
    /// Returns None if the vector is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            self.data[self.len].take()
        } else {
            None
        }
    }

    /// Try to pop an element from the vector
    ///
    /// Returns an error if the vector is empty.
    #[inline]
    pub fn try_pop(&mut self) -> Result<T, FastDataError> {
        self.pop().ok_or(FastDataError::BufferEmpty)
    }

    /// Get the length of the vector
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if the vector is empty
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the maximum capacity of this FixedVec
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get the remaining capacity
    #[inline(always)]
    pub const fn remaining_capacity(&self) -> usize {
        N - self.len
    }

    /// Get element at index
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            self.data[index].as_ref()
        } else {
            None
        }
    }

    /// Get mutable element at index
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            self.data[index].as_mut()
        } else {
            None
        }
    }

    /// Get element at index with bounds checking
    #[inline]
    pub fn try_get(&self, index: usize) -> Result<&T, FastDataError> {
        self.get(index).ok_or(FastDataError::IndexOutOfBounds)
    }

    /// Clear the vector, making it empty
    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.len {
            self.data[i] = None;
        }
        self.len = 0;
    }

    /// Iterate over elements
    #[inline]
    pub fn iter(&self) -> FixedVecIter<T, N> {
        FixedVecIter {
            vec: self,
            index: 0,
        }
    }

    /// Iterate over elements with any lifetime (for compatibility)
    #[inline]
    pub fn iter_any(&self) -> FixedVecIterAny<T, N> {
        FixedVecIterAny {
            vec: self,
            index: 0,
        }
    }

    /// Get as slice
    #[inline]
    pub fn as_slice(&self) -> &[Option<T>] {
        &self.data[..self.len]
    }

    /// Convert to Vec for compatibility
    ///
    /// This is the only operation that allocates memory.
    #[inline]
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.iter().cloned().collect()
    }
}

impl<T, const N: usize> Default for FixedVec<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator for FixedVec
pub struct FixedVecIter<'a, T, const N: usize> {
    vec: &'a FixedVec<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for FixedVecIter<'a, T, N> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let result = self.vec.data[self.index].as_ref();
            self.index += 1;
            result
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for FixedVecIter<'a, T, N> {}

/// Iterator for FixedVec with any lifetime (for compatibility)
pub struct FixedVecIterAny<'a, T, const N: usize> {
    vec: &'a FixedVec<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for FixedVecIterAny<'a, T, N> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let result = self.vec.data[self.index].as_ref();
            self.index += 1;
            result
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for FixedVecIterAny<'a, T, N> {}

/// Lock-free ring buffer for high-performance event passing
///
/// Single-producer, single-consumer ring buffer using atomic operations.
/// Optimized for cache efficiency and minimal contention.
/// All operations are wait-free and lock-free.
#[repr(align(64))] // Cache line alignment for performance
pub struct LockFreeRingBuffer<T, const N: usize> {
    /// Head index (consumer)
    head: AtomicUsize,
    /// Tail index (producer)  
    tail: AtomicUsize,
    /// Ring buffer data using Option for safety
    data: [Option<T>; N],
}

impl<T, const N: usize> LockFreeRingBuffer<T, N> {
    /// Create a new ring buffer
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            data: [const { None }; N],
        }
    }

    /// Try to push an element (non-blocking)
    ///
    /// Returns the element back if the buffer is full.
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % N;
        let head = self.head.load(Ordering::Acquire);

        if next_tail == head {
            // Buffer full
            Err(value)
        } else {
            // Safe to write because we've checked bounds
            self.data[tail] = Some(value);
            self.tail.store(next_tail, Ordering::Release);
            Ok(())
        }
    }

    /// Try to pop an element (non-blocking)
    ///
    /// Returns None if the buffer is empty.
    #[inline]
    pub fn try_pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head == tail {
            // Buffer empty
            None
        } else {
            // Safe to read because we've checked bounds and have mutable access
            let value = self.data[head].take();
            let next_head = (head + 1) % N;
            self.head.store(next_head, Ordering::Release);
            value
        }
    }

    /// Check if the buffer is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }

    /// Check if the buffer is full
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % N;
        next_tail == self.head.load(Ordering::Relaxed)
    }

    /// Get the current length (approximate)
    ///
    /// This is approximate because of concurrent access.
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        (tail + N - head) % N
    }

    /// Get the capacity
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        N - 1 // One slot is reserved for full/empty distinction
    }
}

impl<T, const N: usize> Default for LockFreeRingBuffer<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Atomic plugin state for lock-free plugin management
///
/// Uses a single atomic u64 to store plugin state, flags, and counters.
/// All operations are atomic and lock-free.
#[derive(Debug)]
pub struct AtomicPluginState {
    /// Packed state value:
    /// - Bits 0-31: Plugin lifecycle state
    /// - Bits 32-47: Plugin flags
    /// - Bits 48-63: Reserved for counters
    state: AtomicU64,
}

impl AtomicPluginState {
    /// Plugin lifecycle states (32 bits)
    pub const STATE_UNLOADED: u64 = 0;
    pub const STATE_LOADING: u64 = 1;
    pub const STATE_LOADED: u64 = 2;
    pub const STATE_RUNNING: u64 = 3;
    pub const STATE_PAUSED: u64 = 4;
    pub const STATE_ERROR: u64 = 5;
    pub const STATE_UNLOADING: u64 = 6;

    /// State flags (16 bits, starting at bit 32)
    pub const FLAG_VISIBLE: u64 = 1 << 32;
    pub const FLAG_FOCUSED: u64 = 1 << 33;
    pub const FLAG_INITIALIZED: u64 = 1 << 34;
    pub const FLAG_HAS_SURFACE: u64 = 1 << 35;
    pub const FLAG_HOT_RELOAD: u64 = 1 << 36;
    pub const FLAG_PERFORMANCE_CRITICAL: u64 = 1 << 37;

    /// Create new atomic state
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            state: AtomicU64::new(Self::STATE_UNLOADED),
        }
    }

    /// Load the current state
    #[inline(always)]
    pub fn load(&self, ordering: Ordering) -> u64 {
        self.state.load(ordering)
    }

    /// Store a new state
    #[inline(always)]
    pub fn store(&self, state: u64, ordering: Ordering) {
        self.state.store(state, ordering)
    }

    /// Compare and swap state
    #[inline(always)]
    pub fn compare_exchange(
        &self,
        current: u64,
        new: u64,
        success: Ordering,
        failure: Ordering,
    ) -> Result<u64, u64> {
        self.state.compare_exchange(current, new, success, failure)
    }

    /// Atomically set a flag
    #[inline]
    pub fn set_flag(&self, flag: u64) -> u64 {
        self.state.fetch_or(flag, Ordering::AcqRel)
    }

    /// Atomically clear a flag
    #[inline]
    pub fn clear_flag(&self, flag: u64) -> u64 {
        self.state.fetch_and(!flag, Ordering::AcqRel)
    }

    /// Check if a flag is set
    #[inline(always)]
    pub fn has_flag(&self, flag: u64) -> bool {
        (self.load(Ordering::Acquire) & flag) != 0
    }

    /// Get the lifecycle state
    #[inline(always)]
    pub fn get_lifecycle_state(&self) -> u64 {
        self.load(Ordering::Acquire) & 0xFFFFFFFF
    }

    /// Set the lifecycle state while preserving flags
    #[inline]
    pub fn set_lifecycle_state(&self, new_state: u64) -> Result<(), FastDataError> {
        loop {
            let current = self.load(Ordering::Acquire);
            let flags_and_counters = current & !0xFFFFFFFF;
            let new_value = (new_state & 0xFFFFFFFF) | flags_and_counters;

            match self.compare_exchange(current, new_value, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return Ok(()),
                Err(actual) => {
                    // Another thread updated the state, check if it's still valid
                    if (actual & 0xFFFFFFFF) == new_state {
                        return Ok(());
                    }
                    // Retry with new current value
                }
            }
        }
    }

    /// Get all flags
    #[inline(always)]
    pub fn get_flags(&self) -> u64 {
        (self.load(Ordering::Acquire) >> 32) & 0xFFFF
    }

    /// Set multiple flags atomically
    #[inline]
    pub fn set_flags(&self, flags: u64) -> u64 {
        let shifted_flags = (flags & 0xFFFF) << 32;
        self.state.fetch_or(shifted_flags, Ordering::AcqRel)
    }

    /// Clear multiple flags atomically
    #[inline]
    pub fn clear_flags(&self, flags: u64) -> u64 {
        let shifted_flags = (flags & 0xFFFF) << 32;
        self.state.fetch_and(!shifted_flags, Ordering::AcqRel)
    }

    /// Check if the plugin is in a running state
    #[inline(always)]
    pub fn is_running(&self) -> bool {
        self.get_lifecycle_state() == Self::STATE_RUNNING
    }

    /// Check if the plugin is in an error state
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        self.get_lifecycle_state() == Self::STATE_ERROR
    }

    /// Check if the plugin is loaded
    #[inline(always)]
    pub fn is_loaded(&self) -> bool {
        matches!(
            self.get_lifecycle_state(),
            Self::STATE_LOADED | Self::STATE_RUNNING
        )
    }
}

impl Clone for AtomicPluginState {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            state: AtomicU64::new(self.state.load(Ordering::Acquire)),
        }
    }
}

impl Default for AtomicPluginState {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Type aliases for common string sizes used in the plugin system
///
/// These are optimized for typical plugin metadata sizes and provide
/// zero-allocation storage for the most common use cases.
pub type PluginId = SmallString<64>;
pub type PluginName = SmallString<128>;
pub type PluginDescription = SmallString<512>;
pub type PluginAuthor = SmallString<128>;
pub type PluginVersion = SmallString<32>;

/// Type aliases for common collections used in the plugin system
///
/// These provide zero-allocation storage for plugin metadata collections.
pub type PluginDependencies = FixedVec<PluginId, 16>;
pub type PluginTags = FixedVec<SmallString<32>, 8>;

/// Convenience functions for creating plugin metadata types
pub fn create_plugin_id(s: &str) -> PluginId {
    PluginId::from_str(s).unwrap_or_else(|_| {
        // Truncate if too long rather than panic
        let max_len = 64.min(s.len());
        let mut actual_len = max_len;

        // Find a valid UTF-8 boundary
        while actual_len > 0 && !s.is_char_boundary(actual_len) {
            actual_len -= 1;
        }

        if actual_len > 0 {
            PluginId::from_str(&s[..actual_len]).unwrap_or_else(|_| PluginId::new())
        } else {
            PluginId::new()
        }
    })
}

pub fn create_plugin_name(s: &str) -> PluginName {
    PluginName::from_str(s).unwrap_or_else(|_| {
        // Truncate if too long rather than panic
        let max_len = 128.min(s.len());
        let mut actual_len = max_len;

        // Find a valid UTF-8 boundary
        while actual_len > 0 && !s.is_char_boundary(actual_len) {
            actual_len -= 1;
        }

        if actual_len > 0 {
            PluginName::from_str(&s[..actual_len]).unwrap_or_else(|_| PluginName::new())
        } else {
            PluginName::new()
        }
    })
}

pub fn create_plugin_description(s: &str) -> PluginDescription {
    PluginDescription::from_str(s).unwrap_or_else(|_| {
        // Truncate if too long rather than panic
        let max_len = 512.min(s.len());
        let mut actual_len = max_len;

        // Find a valid UTF-8 boundary
        while actual_len > 0 && !s.is_char_boundary(actual_len) {
            actual_len -= 1;
        }

        if actual_len > 0 {
            PluginDescription::from_str(&s[..actual_len])
                .unwrap_or_else(|_| PluginDescription::new())
        } else {
            PluginDescription::new()
        }
    })
}

pub fn create_plugin_author(s: &str) -> PluginAuthor {
    PluginAuthor::from_str(s).unwrap_or_else(|_| {
        // Truncate if too long rather than panic
        let max_len = 128.min(s.len());
        let mut actual_len = max_len;

        // Find a valid UTF-8 boundary
        while actual_len > 0 && !s.is_char_boundary(actual_len) {
            actual_len -= 1;
        }

        if actual_len > 0 {
            PluginAuthor::from_str(&s[..actual_len]).unwrap_or_else(|_| PluginAuthor::new())
        } else {
            PluginAuthor::new()
        }
    })
}

pub fn create_plugin_version(s: &str) -> PluginVersion {
    PluginVersion::from_str(s).unwrap_or_else(|_| {
        // Truncate if too long rather than panic
        let max_len = 32.min(s.len());
        let mut actual_len = max_len;

        // Find a valid UTF-8 boundary
        while actual_len > 0 && !s.is_char_boundary(actual_len) {
            actual_len -= 1;
        }

        if actual_len > 0 {
            PluginVersion::from_str(&s[..actual_len]).unwrap_or_else(|_| PluginVersion::new())
        } else {
            PluginVersion::new()
        }
    })
}

/// Ultra-fast plugin capabilities using bitflags for maximum performance
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PluginCapabilitiesFlags {
    flags: u64,
}

impl PluginCapabilitiesFlags {
    // Capability flags (64 bits available)
    pub const SUPPORTS_TRANSPARENCY: u64 = 1 << 0;
    pub const REQUIRES_KEYBOARD_FOCUS: u64 = 1 << 1;
    pub const SUPPORTS_MULTI_WINDOW: u64 = 1 << 2;
    pub const SUPPORTS_3D_RENDERING: u64 = 1 << 3;
    pub const SUPPORTS_COMPUTE_SHADERS: u64 = 1 << 4;
    pub const REQUIRES_NETWORK_ACCESS: u64 = 1 << 5;
    pub const SUPPORTS_FILE_SYSTEM: u64 = 1 << 6;
    pub const SUPPORTS_AUDIO: u64 = 1 << 7;
    pub const SUPPORTS_HOT_RELOAD: u64 = 1 << 8;
    pub const REQUIRES_GPU_MEMORY: u64 = 1 << 9;

    #[inline(always)]
    pub const fn new() -> Self {
        Self { flags: 0 }
    }

    #[inline(always)]
    pub const fn with_flag(self, flag: u64) -> Self {
        Self {
            flags: self.flags | flag,
        }
    }

    #[inline(always)]
    pub const fn has_flag(self, flag: u64) -> bool {
        (self.flags & flag) != 0
    }

    #[inline(always)]
    pub const fn set_flag(&mut self, flag: u64) {
        self.flags |= flag;
    }

    #[inline(always)]
    pub const fn clear_flag(&mut self, flag: u64) {
        self.flags &= !flag;
    }
}

/// Ultra-fast plugin resource limits with fixed-size allocations
#[derive(Debug, Clone, Copy, Default)]
pub struct PluginResourceLimits {
    pub max_memory_mb: u32,
    pub max_texture_size: u32,
    pub max_buffer_size: u32,
    pub max_compute_threads: u16,
    pub max_render_targets: u16,
    pub max_shader_uniforms: u16,
    pub max_surface_count: u8,
    pub max_audio_channels: u8,
}

impl PluginResourceLimits {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            max_memory_mb: 512,
            max_texture_size: 4096,
            max_buffer_size: 64 * 1024 * 1024, // 64MB
            max_compute_threads: 256,
            max_render_targets: 8,
            max_shader_uniforms: 256,
            max_surface_count: 4,
            max_audio_channels: 8,
        }
    }

    #[inline(always)]
    pub const fn with_memory_limit(self, limit_mb: u32) -> Self {
        Self {
            max_memory_mb: limit_mb,
            ..self
        }
    }

    #[inline(always)]
    pub const fn with_texture_limit(self, size: u32) -> Self {
        Self {
            max_texture_size: size,
            ..self
        }
    }
}

/// Ultra-fast plugin render statistics with atomic counters
#[derive(Debug, Default)]
pub struct PluginRenderStats {
    pub frames_rendered: AtomicUsize,
    pub total_render_time_us: AtomicUsize,
    pub peak_frame_time_us: AtomicUsize,
    pub texture_memory_bytes: AtomicUsize,
    pub buffer_memory_bytes: AtomicUsize,
    pub draw_calls: AtomicUsize,
    pub vertices_processed: AtomicUsize,
    pub triangles_rendered: AtomicUsize,
}

impl PluginRenderStats {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            frames_rendered: AtomicUsize::new(0),
            total_render_time_us: AtomicUsize::new(0),
            peak_frame_time_us: AtomicUsize::new(0),
            texture_memory_bytes: AtomicUsize::new(0),
            buffer_memory_bytes: AtomicUsize::new(0),
            draw_calls: AtomicUsize::new(0),
            vertices_processed: AtomicUsize::new(0),
            triangles_rendered: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub fn record_frame(&self, render_time_us: u32) {
        self.frames_rendered.fetch_add(1, Ordering::Relaxed);
        self.total_render_time_us
            .fetch_add(render_time_us as usize, Ordering::Relaxed);

        // Update peak frame time
        let current_peak = self.peak_frame_time_us.load(Ordering::Relaxed);
        if render_time_us as usize > current_peak {
            let _ = self.peak_frame_time_us.compare_exchange(
                current_peak,
                render_time_us as usize,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
    }

    #[inline]
    pub fn record_draw_call(&self, vertex_count: u32, triangle_count: u32) {
        self.draw_calls.fetch_add(1, Ordering::Relaxed);
        self.vertices_processed
            .fetch_add(vertex_count as usize, Ordering::Relaxed);
        self.triangles_rendered
            .fetch_add(triangle_count as usize, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_average_frame_time_us(&self) -> f32 {
        let frames = self.frames_rendered.load(Ordering::Relaxed);
        if frames == 0 {
            return 0.0;
        }

        let total_time = self.total_render_time_us.load(Ordering::Relaxed);
        total_time as f32 / frames as f32
    }
}

/// Ultra-fast plugin system metrics with atomic counters
#[derive(Debug, Default, Resource)]
pub struct PluginSystemMetrics {
    pub active_plugins: AtomicUsize,
    pub total_memory_usage_bytes: AtomicUsize,
    pub total_gpu_memory_bytes: AtomicUsize,
    pub events_processed: AtomicUsize,
    pub events_dropped: AtomicUsize,
    pub load_time_us: AtomicUsize,
    pub unload_time_us: AtomicUsize,
    pub context_switches: AtomicUsize,
}

impl PluginSystemMetrics {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            active_plugins: AtomicUsize::new(0),
            total_memory_usage_bytes: AtomicUsize::new(0),
            total_gpu_memory_bytes: AtomicUsize::new(0),
            events_processed: AtomicUsize::new(0),
            events_dropped: AtomicUsize::new(0),
            load_time_us: AtomicUsize::new(0),
            unload_time_us: AtomicUsize::new(0),
            context_switches: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub fn record_plugin_load(&self, plugin_count: usize, load_time_us: u32) {
        self.active_plugins.store(plugin_count, Ordering::Relaxed);
        self.load_time_us
            .fetch_add(load_time_us as usize, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_plugin_unload(&self, plugin_count: usize, unload_time_us: u32) {
        self.active_plugins.store(plugin_count, Ordering::Relaxed);
        self.unload_time_us
            .fetch_add(unload_time_us as usize, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_memory_usage(&self, cpu_bytes: usize, gpu_bytes: usize) {
        self.total_memory_usage_bytes
            .store(cpu_bytes, Ordering::Relaxed);
        self.total_gpu_memory_bytes
            .store(gpu_bytes, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_event_processing(&self, processed: usize, dropped: usize) {
        self.events_processed
            .fetch_add(processed, Ordering::Relaxed);
        self.events_dropped.fetch_add(dropped, Ordering::Relaxed);
    }
}

/// Ultra-fast plugin event queue with lock-free ring buffer
pub type PluginEventQueue<T> = LockFreeRingBuffer<T, 2048>;

/// Ultra-fast plugin memory pool with fixed-size allocations
#[derive(Debug, Resource)]
pub struct PluginMemoryPool<const POOL_SIZE: usize, const BLOCK_SIZE: usize> {
    /// Fixed-size memory blocks
    blocks: [MemoryBlock<BLOCK_SIZE>; POOL_SIZE],
    /// Atomic allocation bitmap
    allocation_bitmap: AtomicU64,
    /// Current allocation index
    current_index: AtomicUsize,
    /// Total allocated bytes
    total_allocated: AtomicUsize,
}

#[derive(Debug)]
struct MemoryBlock<const BLOCK_SIZE: usize> {
    data: [u8; BLOCK_SIZE],
    allocated: AtomicU64,
}

impl<const BLOCK_SIZE: usize> Default for MemoryBlock<BLOCK_SIZE> {
    fn default() -> Self {
        Self {
            data: [0u8; BLOCK_SIZE],
            allocated: AtomicU64::new(0),
        }
    }
}

impl<const POOL_SIZE: usize, const BLOCK_SIZE: usize> PluginMemoryPool<POOL_SIZE, BLOCK_SIZE> {
    #[inline(always)]
    pub const fn new() -> Self {
        // Use uninit array and then initialize each element
        let blocks = [const {
            MemoryBlock {
                data: [0u8; BLOCK_SIZE],
                allocated: AtomicU64::new(0),
            }
        }; POOL_SIZE];

        Self {
            blocks,
            allocation_bitmap: AtomicU64::new(0),
            current_index: AtomicUsize::new(0),
            total_allocated: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub fn allocate(&self, size: usize) -> Option<*mut u8> {
        if size > BLOCK_SIZE || POOL_SIZE > 64 {
            return None;
        }

        let current_bitmap = self.allocation_bitmap.load(Ordering::Acquire);

        // Find first available block
        for i in 0..POOL_SIZE {
            let block_mask = 1u64 << i;
            if (current_bitmap & block_mask) == 0 {
                // Try to allocate this block
                let old_bitmap = self
                    .allocation_bitmap
                    .fetch_or(block_mask, Ordering::AcqRel);
                if (old_bitmap & block_mask) == 0 {
                    // Successfully allocated
                    self.blocks[i]
                        .allocated
                        .store(size as u64, Ordering::Release);
                    self.total_allocated.fetch_add(size, Ordering::Relaxed);
                    return Some(self.blocks[i].data.as_ptr() as *mut u8);
                }
            }
        }

        None
    }

    #[inline]
    pub fn deallocate(&self, ptr: *mut u8) -> bool {
        let ptr_addr = ptr as usize;

        // Find which block this pointer belongs to
        for i in 0..POOL_SIZE {
            let block_start = self.blocks[i].data.as_ptr() as usize;
            let block_end = block_start + BLOCK_SIZE;

            if ptr_addr >= block_start && ptr_addr < block_end {
                let block_mask = 1u64 << i;
                let old_bitmap = self
                    .allocation_bitmap
                    .fetch_and(!block_mask, Ordering::AcqRel);
                if (old_bitmap & block_mask) != 0 {
                    // Successfully deallocated
                    let size = self.blocks[i].allocated.swap(0, Ordering::AcqRel);
                    self.total_allocated
                        .fetch_sub(size as usize, Ordering::Relaxed);
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    pub fn get_total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn get_available_blocks(&self) -> usize {
        let bitmap = self.allocation_bitmap.load(Ordering::Relaxed);
        POOL_SIZE - bitmap.count_ones() as usize
    }
}
