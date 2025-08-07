//! Utility types and functions for the XREAL plugin system

use std::{
    fmt,
    ops::Deref,
    str::FromStr,
};

/// A small string optimized for short strings with a fixed-size stack allocation
/// and a fallback to heap allocation for longer strings.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SmallString<const N: usize> {
    inner: SmallStringInner<N>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum SmallStringInner<const N: usize> {
    Inline { len: u8, data: [u8; N] },
    Heap(String),
}

impl<const N: usize> Default for SmallString<N> {
    fn default() -> Self {
        Self {
            inner: SmallStringInner::Inline {
                len: 0,
                data: [0; N],
            },
        }
    }
}

impl<const N: usize> SmallString<N> {
    /// Creates a new empty `SmallString`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `SmallString` from a string slice.
    pub fn from_str(s: &str) -> Self {
        if s.len() <= N {
            let mut data = [0u8; N];
            data[..s.len()].copy_from_slice(s.as_bytes());
            Self {
                inner: SmallStringInner::Inline {
                    len: s.len() as u8,
                    data,
                },
            }
        } else {
            Self {
                inner: SmallStringInner::Heap(s.to_string()),
            }
        }
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        match &self.inner {
            SmallStringInner::Inline { len, .. } => *len as usize,
            SmallStringInner::Heap(s) => s.len(),
        }
    }

    /// Returns `true` if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts the `SmallString` into a `String`.
    pub fn into_string(self) -> String {
        match self.inner {
            SmallStringInner::Inline { len, data } => {
                String::from_utf8_lossy(&data[..len as usize]).into_owned()
            }
            SmallStringInner::Heap(s) => s,
        }
    }

    /// Returns a string slice of the entire string.
    pub fn as_str(&self) -> &str {
        match &self.inner {
            SmallStringInner::Inline { len, data } => {
                // SAFETY: We only store valid UTF-8 in the inline buffer
                unsafe { std::str::from_utf8_unchecked(&data[..*len as usize]) }
            }
            SmallStringInner::Heap(s) => s.as_str(),
        }
    }
}

impl<const N: usize> From<&str> for SmallString<N> {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl<const N: usize> From<String> for SmallString<N> {
    fn from(s: String) -> Self {
        if s.len() <= N {
            let mut data = [0u8; N];
            data[..s.len()].copy_from_slice(s.as_bytes());
            Self {
                inner: SmallStringInner::Inline {
                    len: s.len() as u8,
                    data,
                },
            }
        } else {
            Self {
                inner: SmallStringInner::Heap(s),
            }
        }
    }
}

impl<const N: usize> Deref for SmallString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for SmallString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> FromStr for SmallString<N> {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
}

impl<const N: usize> AsRef<str> for SmallString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A fixed-size vector with compile-time capacity checking
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FixedVec<T, const N: usize> {
    data: Vec<T>,
    _marker: std::marker::PhantomData<[T; N]>,
}

impl<T, const N: usize> FixedVec<T, N> {
    /// Creates a new empty `FixedVec`.
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(N),
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Pushes an element to the vector.
    ///
    /// # Panics
    ///
    /// Panics if the vector is full.
    pub fn push(&mut self, value: T) {
        if self.data.len() >= N {
            panic!("FixedVec capacity exceeded");
        }
        self.data.push(value);
    }

    /// Returns a reference to the element at the given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Returns a mutable reference to the element at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}

impl<T, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> std::ops::Deref for FixedVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T, const N: usize> std::ops::DerefMut for FixedVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A fixed-size hash map with compile-time capacity checking
#[derive(Debug, Clone)]
pub struct FixedHashMap<K, V, const N: usize>
where
    K: std::hash::Hash + Eq,
{
    inner: std::collections::HashMap<K, V>,
    _marker: std::marker::PhantomData<[(); N]>,
}

impl<K, V, const N: usize> FixedHashMap<K, V, N>
where
    K: std::hash::Hash + Eq,
{
    /// Creates a new empty `FixedHashMap`.
    pub fn new() -> Self {
        Self {
            inner: std::collections::HashMap::with_capacity(N),
            _marker: std::marker::PhantomData,
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// # Panics
    ///
    /// Panics if the map is full.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.inner.len() >= N && !self.inner.contains_key(&key) {
            panic!("FixedHashMap capacity exceeded");
        }
        self.inner.insert(key, value)
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V, const N: usize> Default for FixedHashMap<K, V, N>
where
    K: std::hash::Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}
/// A set of plugin dependencies with compile-time capacity checking
#[derive(Debug, Clone, Default)]
pub struct PluginDependencies<const N: usize> {
    inner: FixedVec<SmallString<64>, N>,
}

impl<const N: usize> PluginDependencies<N> {
    /// Creates a new empty `PluginDependencies`.
    pub fn new() -> Self {
        Self {
            inner: FixedVec::new(),
        }
    }

    /// Adds a dependency to the set.
    pub fn add(&mut self, dependency: impl Into<SmallString<64>>) -> bool {
        if self.inner.len() < N {
            self.inner.push(dependency.into());
            true
        } else {
            false
        }
    }

    /// Returns an iterator over the dependencies.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().map(|s| s.as_str())
    }

    /// Returns the number of dependencies.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there are no dependencies.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
/// A set of plugin tags with compile-time capacity checking
#[derive(Debug, Clone, Default)]
pub struct PluginTags<const N: usize> {
    inner: FixedVec<SmallString<32>, N>,
}

impl<const N: usize> PluginTags<N> {
    /// Creates a new empty `PluginTags`.
    pub fn new() -> Self {
        Self {
            inner: FixedVec::new(),
        }
    }

    /// Adds a tag to the set.
    pub fn add(&mut self, tag: impl Into<SmallString<32>>) -> bool {
        if self.inner.len() < N {
            self.inner.push(tag.into());
            true
        } else {
            false
        }
    }

    /// Returns an iterator over the tags.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().map(|s| s.as_str())
    }

    /// Returns the number of tags.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there are no tags.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
/// Thread-safe plugin state for atomic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AtomicPluginState {
    /// Plugin is currently loading
    Loading,
    /// Plugin is running normally
    Running,
    /// Plugin is paused
    Paused,
    /// Plugin encountered an error
    Error,
    /// Plugin is being unloaded
    Unloading,
    /// Plugin state constants for compatibility
    StateLoaded,
    StateUnloaded,
    StateRunning,
    StatePaused,
    StateError,
    /// Performance flags
    FlagPerformanceCritical,
    FlagMemoryIntensive,
    FlagInitialized,
    FlagReady,
    FlagActive,
}

impl Default for AtomicPluginState {
    fn default() -> Self {
        Self::Loading
    }
}

impl AtomicPluginState {
    /// Create a new plugin state (compatibility method)
    pub fn new() -> Self {
        Self::Loading
    }

    /// Set a flag on the plugin state (compatibility method)
    pub fn set_flag(&mut self, _flag: AtomicPluginState) {
        // This is a compatibility method for the existing code
        // In a real implementation, this would manage state flags
    }

    /// Clear a flag on the plugin state (compatibility method)
    pub fn clear_flag(&mut self, _flag: AtomicPluginState) {
        // This is a compatibility method for the existing code
        // In a real implementation, this would clear state flags
    }

    /// Set the lifecycle state (compatibility method)
    pub fn set_lifecycle_state(&mut self, state: AtomicPluginState) {
        *self = state;
    }

    /// Get the current lifecycle state (compatibility method)
    pub fn get_lifecycle_state(&self) -> AtomicPluginState {
        *self
    }
}

/// A thread-safe queue for plugin events with compile-time capacity checking
#[derive(Debug)]
pub struct PluginEventQueue<T, const N: usize> {
    inner: crossbeam::queue::ArrayQueue<T>,
}

impl<T, const N: usize> PluginEventQueue<T, N> {
    /// Creates a new empty `PluginEventQueue`.
    pub fn new() -> Self {
        Self {
            inner: crossbeam::queue::ArrayQueue::new(N),
        }
    }

    /// Pushes an event to the queue.
    /// Returns `true` if the event was pushed successfully, `false` if the queue is full.
    pub fn push(&self, event: T) -> bool {
        self.inner.push(event).is_ok()
    }

    /// Tries to push an event to the queue.
    /// Returns `Ok(())` if successful, `Err(event)` if the queue is full.
    pub fn try_push(&self, event: T) -> Result<(), T> {
        self.inner.push(event).map_err(|e| e)
    }

    /// Pops an event from the queue.
    /// Returns `None` if the queue is empty.
    pub fn pop(&self) -> Option<T> {
        self.inner.pop()
    }

    /// Tries to pop an event from the queue.
    /// Returns `Some(event)` if successful, `None` if the queue is empty.
    pub fn try_pop(&self) -> Option<T> {
        self.inner.pop()
    }

    /// Returns the number of events in the queue.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T, const N: usize> Default for PluginEventQueue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
/// Type aliases for plugin system components
pub type PluginId = SmallString<64>;
pub type PluginName = SmallString<128>;
pub type PluginVersion = SmallString<32>;
pub type PluginDescription = SmallString<256>;
pub type PluginAuthor = SmallString<64>;

/// Resource limits for plugin system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginResourceLimits {
    /// Memory limit in MB
    pub memory_limit_mb: u32,
    /// Texture memory limit in MB
    pub texture_limit_mb: u32,
    /// Maximum number of threads
    pub max_threads: u32,
    /// Maximum file handles
    pub max_file_handles: u32,
}

impl PluginResourceLimits {
    /// Create new resource limits with default values
    pub fn new() -> Self {
        Self {
            memory_limit_mb: 256,
            texture_limit_mb: 128,
            max_threads: 4,
            max_file_handles: 64,
        }
    }

    /// Set memory limit in MB
    pub fn with_memory_limit(mut self, limit_mb: u32) -> Self {
        self.memory_limit_mb = limit_mb;
        self
    }

    /// Set texture memory limit in MB
    pub fn with_texture_limit(mut self, limit_mb: u32) -> Self {
        self.texture_limit_mb = limit_mb;
        self
    }

    /// Set maximum number of threads
    pub fn with_max_threads(mut self, max_threads: u32) -> Self {
        self.max_threads = max_threads;
        self
    }

    /// Set maximum file handles
    pub fn with_max_file_handles(mut self, max_handles: u32) -> Self {
        self.max_file_handles = max_handles;
        self
    }
}

impl Default for PluginResourceLimits {
    fn default() -> Self {
        Self::new()
    }
}
/// Plugin rendering statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PluginRenderStats {
    /// Number of frames rendered
    pub frames_rendered: u64,
    /// Average frame time in milliseconds
    pub avg_frame_time_ms: f32,
    /// Peak frame time in milliseconds
    pub peak_frame_time_ms: f32,
    /// Total GPU memory used in bytes
    pub gpu_memory_used: u64,
    /// Number of draw calls
    pub draw_calls: u32,
    /// Number of vertices rendered
    pub vertices_rendered: u64,
}

impl PluginRenderStats {
    /// Create new render stats with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Update frame time statistics
    pub fn update_frame_time(&mut self, frame_time_ms: f32) {
        self.frames_rendered += 1;

        // Update average using exponential moving average
        let alpha = 0.1;
        self.avg_frame_time_ms = self.avg_frame_time_ms * (1.0 - alpha) + frame_time_ms * alpha;

        // Update peak
        if frame_time_ms > self.peak_frame_time_ms {
            self.peak_frame_time_ms = frame_time_ms;
        }
    }

    /// Update GPU memory usage
    pub fn update_gpu_memory(&mut self, memory_bytes: u64) {
        self.gpu_memory_used = memory_bytes;
    }

    /// Record draw call statistics
    pub fn record_draw_call(&mut self, vertex_count: u64) {
        self.draw_calls += 1;
        self.vertices_rendered += vertex_count;
    }
}
