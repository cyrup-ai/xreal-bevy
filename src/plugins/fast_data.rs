//! Ultra-Fast Zero-Allocation Data Structures for Plugin System
//! 
//! This module provides blazing-fast, zero-allocation data structures optimized for
//! the XREAL plugin system. All structures are designed for maximum performance
//! with no locking, no heap allocations, and cache-optimized memory layouts.

use core::{
    mem::{self, MaybeUninit},
    ptr,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    fmt,
    hash::{Hash, Hasher},
};

/// Ultra-fast small string with zero heap allocations
/// 
/// Stores strings up to N bytes directly inline. For strings exceeding N bytes,
/// stores a reference to a static string. This eliminates all heap allocations
/// for typical plugin metadata strings.
#[derive(Clone, Copy)]
pub struct SmallString<const N: usize> {
    /// Length of the string (high bit indicates if this is a static reference)
    len: u8,
    /// Inline data or pointer to static string
    data: SmallStringData<N>,
}

#[derive(Clone, Copy)]
union SmallStringData<const N: usize> {
    /// Inline storage for small strings
    inline: [u8; N],
    /// Pointer to static string for large strings
    static_ptr: *const u8,
}

// SAFETY: SmallStringData only contains pointers to static strings or inline data,
// both of which are safe to send and share between threads
unsafe impl<const N: usize> Send for SmallStringData<N> {}
unsafe impl<const N: usize> Sync for SmallStringData<N> {}

impl<const N: usize> SmallString<N> {
    /// Create a new SmallString from a static string reference
    /// 
    /// This is zero-cost for static strings and enables compile-time optimization.
    #[inline(always)]
    pub const fn from_static(s: &'static str) -> Self {
        let bytes = s.as_bytes();
        if bytes.len() <= N {
            // Store inline
            let mut inline_data = [0u8; N];
            let mut i = 0;
            while i < bytes.len() {
                inline_data[i] = bytes[i];
                i += 1;
            }
            Self {
                len: bytes.len() as u8,
                data: SmallStringData { inline: inline_data },
            }
        } else {
            // Store as static reference (marked with high bit)
            Self {
                len: (bytes.len() as u8) | 0x80,
                data: SmallStringData { static_ptr: bytes.as_ptr() },
            }
        }
    }
    
    /// Create from a string slice (runtime version)
    #[inline]
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        if bytes.len() <= N {
            let mut inline_data = [0u8; N];
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), inline_data.as_mut_ptr(), bytes.len());
            }
            Self {
                len: bytes.len() as u8,
                data: SmallStringData { inline: inline_data },
            }
        } else {
            // For dynamic strings that are too long, we need to truncate or error
            // Since we want zero allocation, we'll truncate to fit
            let mut inline_data = [0u8; N];
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), inline_data.as_mut_ptr(), N);
            }
            Self {
                len: N as u8,
                data: SmallStringData { inline: inline_data },
            }
        }
    }
    
    /// Get the string as a &str
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        unsafe {
            if (self.len & 0x80) != 0 {
                // Static string reference
                let actual_len = (self.len & 0x7F) as usize;
                let slice = core::slice::from_raw_parts(self.data.static_ptr, actual_len);
                core::str::from_utf8_unchecked(slice)
            } else {
                // Inline string
                let slice = core::slice::from_raw_parts(
                    self.data.inline.as_ptr(),
                    self.len as usize
                );
                core::str::from_utf8_unchecked(slice)
            }
        }
    }
    
    /// Get the length of the string
    #[inline(always)]
    pub const fn len(&self) -> usize {
        if (self.len & 0x80) != 0 {
            (self.len & 0x7F) as usize
        } else {
            self.len as usize
        }
    }
    
    /// Check if the string is empty
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<const N: usize> fmt::Display for SmallString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> fmt::Debug for SmallString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SmallString(\"{}\")", self.as_str())
    }
}

impl<const N: usize> PartialEq for SmallString<N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> Eq for SmallString<N> {}

impl<const N: usize> Hash for SmallString<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<const N: usize> PartialEq<&str> for SmallString<N> {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Ultra-fast fixed-capacity vector with zero heap allocations
/// 
/// Stores up to N elements directly inline with O(1) operations.
/// Perfect for plugin dependencies, capabilities, etc.
pub struct FixedVec<T, const N: usize> {
    /// Number of valid elements
    len: usize,
    /// Inline storage for elements
    data: [MaybeUninit<T>; N],
}

impl<T: Clone, const N: usize> Clone for FixedVec<T, N> {
    fn clone(&self) -> Self {
        let mut new_data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        
        // Clone only the valid elements
        for i in 0..self.len {
            new_data[i] = MaybeUninit::new(unsafe { self.data[i].assume_init_ref().clone() });
        }
        
        Self {
            len: self.len,
            data: new_data,
        }
    }
}

impl<T: Copy, const N: usize> FixedVec<T, N> {
    /// Create a new empty FixedVec
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
    
    /// Push an element (returns false if full)
    #[inline]
    pub fn push(&mut self, value: T) -> bool {
        if self.len < N {
            self.data[self.len] = MaybeUninit::new(value);
            self.len += 1;
            true
        } else {
            false
        }
    }
    
    /// Get the length
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }
    
    /// Check if empty
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Get element at index
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe { self.data[index].assume_init_ref() })
        } else {
            None
        }
    }
    
    /// Iterate over elements
    #[inline]
    pub fn iter(&self) -> FixedVecIter<T, N> {
        FixedVecIter {
            vec: self,
            index: 0,
        }
    }
    
    /// Get as slice
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(
                self.data.as_ptr() as *const T,
                self.len
            )
        }
    }
}

impl<T: Copy, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

// Implementation for any type T (not just Copy)
impl<T, const N: usize> FixedVec<T, N> {
    /// Push an element for any type (not just Copy)
    #[inline]
    pub fn push_any(&mut self, value: T) -> Result<(), T> {
        if self.len < N {
            self.data[self.len] = MaybeUninit::new(value);
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }
    
    /// Iterate over elements for any type
    #[inline]
    pub fn iter_any(&self) -> FixedVecIterAny<T, N> {
        FixedVecIterAny {
            vec: self,
            index: 0,
        }
    }
}

/// Iterator for FixedVec (Copy types only)
pub struct FixedVecIter<'a, T, const N: usize> {
    vec: &'a FixedVec<T, N>,
    index: usize,
}

impl<'a, T: Copy, const N: usize> Iterator for FixedVecIter<'a, T, N> {
    type Item = &'a T;
    
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let item = unsafe { self.vec.data[self.index].assume_init_ref() };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
    
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a, T: Copy, const N: usize> ExactSizeIterator for FixedVecIter<'a, T, N> {}

/// Iterator for FixedVec (any type)
pub struct FixedVecIterAny<'a, T, const N: usize> {
    vec: &'a FixedVec<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for FixedVecIterAny<'a, T, N> {
    type Item = &'a T;
    
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let item = unsafe { self.vec.data[self.index].assume_init_ref() };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
    
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for FixedVecIterAny<'a, T, N> {}

/// Lock-free ring buffer for high-performance event passing
/// 
/// Single-producer, single-consumer ring buffer using atomic operations.
/// Optimized for cache efficiency and minimal contention.
#[repr(align(64))] // Cache line alignment
pub struct LockFreeRingBuffer<T, const N: usize> {
    /// Head index (consumer)
    head: AtomicUsize,
    /// Padding to avoid false sharing
    _pad1: [u8; 64 - mem::size_of::<AtomicUsize>()],
    /// Tail index (producer)  
    tail: AtomicUsize,
    /// Padding to avoid false sharing
    _pad2: [u8; 64 - mem::size_of::<AtomicUsize>()],
    /// Ring buffer data
    data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> LockFreeRingBuffer<T, N> {
    /// Create a new ring buffer
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: AtomicUsize::new(0),
            _pad1: [0; 64 - mem::size_of::<AtomicUsize>()],
            tail: AtomicUsize::new(0),
            _pad2: [0; 64 - mem::size_of::<AtomicUsize>()],
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
    
    /// Try to push an element (non-blocking)
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % N;
        let head = self.head.load(Ordering::Acquire);
        
        if next_tail == head {
            // Buffer full
            Err(value)
        } else {
            unsafe {
                ptr::write(self.data[tail].as_mut_ptr(), value);
            }
            self.tail.store(next_tail, Ordering::Release);
            Ok(())
        }
    }
    
    /// Try to pop an element (non-blocking)
    #[inline]
    pub fn try_pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        
        if head == tail {
            // Buffer empty
            None
        } else {
            let value = unsafe { ptr::read(self.data[head].as_ptr()) };
            let next_head = (head + 1) % N;
            self.head.store(next_head, Ordering::Release);
            Some(value)
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
}

/// Atomic plugin state for lock-free state management
/// 
/// Packs multiple state values into a single 64-bit atomic for efficient updates.
/// Bit layout: [32-bit state][16-bit flags][16-bit counters]
pub struct AtomicPluginState {
    /// Packed state value
    state: AtomicU64,
}

impl Clone for AtomicPluginState {
    fn clone(&self) -> Self {
        Self {
            state: AtomicU64::new(self.state.load(Ordering::Acquire)),
        }
    }
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
    
    /// State flags (16 bits)
    pub const FLAG_VISIBLE: u64 = 1 << 32;
    pub const FLAG_FOCUSED: u64 = 1 << 33;
    pub const FLAG_INITIALIZED: u64 = 1 << 34;
    pub const FLAG_HAS_SURFACE: u64 = 1 << 35;
    
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
    pub fn set_flag(&self, flag: u64) {
        self.state.fetch_or(flag, Ordering::AcqRel);
    }
    
    /// Atomically clear a flag
    #[inline]
    pub fn clear_flag(&self, flag: u64) {
        self.state.fetch_and(!flag, Ordering::AcqRel);
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
    pub fn set_lifecycle_state(&self, new_state: u64) -> Result<(), u64> {
        loop {
            let current = self.load(Ordering::Acquire);
            let flags_and_counters = current & !0xFFFFFFFF;
            let new_value = (new_state & 0xFFFFFFFF) | flags_and_counters;
            
            match self.compare_exchange(
                current,
                new_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
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
}

impl Default for AtomicPluginState {
    fn default() -> Self {
        Self::new()
    }
}

/// Type aliases for common string sizes
pub type PluginId = SmallString<64>;
pub type PluginName = SmallString<128>;
pub type PluginDescription = SmallString<512>;
pub type PluginAuthor = SmallString<128>;
pub type PluginVersion = SmallString<32>;

/// Type aliases for common collections
pub type PluginDependencies = FixedVec<PluginId, 16>;
pub type PluginTags = FixedVec<SmallString<32>, 8>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_small_string_static() {
        const TEST_STR: SmallString<32> = SmallString::from_static("test");
        assert_eq!(TEST_STR.as_str(), "test");
        assert_eq!(TEST_STR.len(), 4);
    }
    
    #[test]
    fn test_fixed_vec() {
        let mut vec = FixedVec::<i32, 4>::new();
        assert!(vec.push(1));
        assert!(vec.push(2));
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), Some(&1));
    }
    
    #[test]
    fn test_ring_buffer() {
        let buffer = LockFreeRingBuffer::<i32, 4>::new();
        assert!(buffer.try_push(1).is_ok());
        assert!(buffer.try_push(2).is_ok());
        assert_eq!(buffer.try_pop(), Some(1));
        assert_eq!(buffer.try_pop(), Some(2));
        assert_eq!(buffer.try_pop(), None);
    }
    
    #[test]
    fn test_atomic_state() {
        let state = AtomicPluginState::new();
        assert_eq!(state.get_lifecycle_state(), AtomicPluginState::STATE_UNLOADED);
        
        state.set_flag(AtomicPluginState::FLAG_VISIBLE);
        assert!(state.has_flag(AtomicPluginState::FLAG_VISIBLE));
        
        assert!(state.set_lifecycle_state(AtomicPluginState::STATE_RUNNING).is_ok());
        assert_eq!(state.get_lifecycle_state(), AtomicPluginState::STATE_RUNNING);
        assert!(state.has_flag(AtomicPluginState::FLAG_VISIBLE));
    }
}