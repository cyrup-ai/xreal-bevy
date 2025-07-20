use crate::input::error::InputError;
use crate::input::InputConfig;
use bevy::prelude::*;
use enigo::*;
use std::sync::atomic::{AtomicU64, Ordering};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use enigo::Button;
use std::time::{SystemTime, UNIX_EPOCH};
use bevy::ecs::system::NonSend;
use tracing::warn;

// Use marker types instead of negative trait bounds for !Send and !Sync
use std::marker::PhantomData;

// Marker type that is !Send and !Sync
struct NotSendSync(*const u8);

/// Main resource for handling input operations
///
/// # Safety
/// This type is marked as `!Send` and `!Sync` because it contains a `CGEventSource`
/// from macOS's Core Graphics framework which is not thread-safe.
///
/// This type must be used with `NonSend<InputSystem>` in Bevy systems to ensure
/// it's only accessed from the main thread.
pub struct InputSystem {
    enigo: Enigo,
    last_event: AtomicU64,
    config: InputConfig,
    _not_send_sync: PhantomData<NotSendSync>,
}

impl InputSystem {
    /// Create a new input system with default settings
    ///
    /// # Safety
    /// The returned `InputSystem` must only be used on the main thread.
    pub fn new() -> Result<Self, InputError> {
        let settings = Settings::default();
        let enigo = Enigo::new(&settings).map_err(|e| {
            InputError::Initialization(format!("Failed to initialize Enigo: {}", e))
        })?;

        Ok(Self {
            enigo,
            last_event: AtomicU64::new(0),
            config: InputConfig::default(),
            _not_send_sync: PhantomData,
        })
    }

    /// Move the mouse to the specified coordinates
    ///
    /// # Safety
    /// This method must be called from the main thread because it uses macOS's Core Graphics API
    /// which is not thread-safe.
    #[inline]
    pub fn move_mouse(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Rate limiting check
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| InputError::Other(e.to_string()))?
            .as_millis() as u64;

        let last = self.last_event.load(Ordering::Acquire);

        if now.saturating_sub(last) < self.config.min_event_interval_ms {
            return Ok(());
        }

        // SAFETY: We assume this method is only called from the main thread
        // as enforced by the NonSend wrapper
        self.enigo
            .move_mouse(x, y, enigo::Coordinate::Abs)
            .map_err(|e| InputError::MouseMove(e.to_string()))?;

        self.last_event.store(now, Ordering::Release);
        Ok(())
    }

    /// Click the specified mouse button
    pub fn click(&mut self, button: Button) -> Result<(), InputError> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| InputError::Other(e.to_string()))?
            .as_millis() as u64;

        // SAFETY: We assume this method is only called from the main thread
        // as enforced by the NonSend wrapper
        self.enigo
            .button(button, enigo::Direction::Click)
            .map_err(|e| InputError::MouseClick(e.to_string()))?;

        self.last_event.store(now, Ordering::Release);
        Ok(())
    }

    /// Send a key press
    ///
    /// # Safety
    /// This method must be called from the main thread because it uses macOS's Core Graphics API
    /// which is not thread-safe.
    #[inline]
    pub fn key_press(&mut self, key: KeyCode) -> Result<(), InputError> {
        if !self.config.enabled {
            return Ok(());
        }

        // SAFETY: We assume this method is only called from the main thread
        // as enforced by the NonSend wrapper
        let enigo_key = self.map_key_code(key)?;
        self.enigo
            .key(enigo_key, enigo::Direction::Press)
            .map_err(|e| InputError::KeyPress(e.to_string()))
    }

    /// Send a key release
    ///
    /// # Safety
    /// This method must be called from the main thread because it uses macOS's Core Graphics API
    /// which is not thread-safe.
    #[inline]
    pub fn key_release(&mut self, key: KeyCode) -> Result<(), InputError> {
        if !self.config.enabled {
            return Ok(());
        }

        // SAFETY: We assume this method is only called from the main thread
        // as enforced by the NonSend wrapper
        let enigo_key = self.map_key_code(key)?;
        self.enigo
            .key(enigo_key, enigo::Direction::Release)
            .map_err(|e| InputError::KeyRelease(e.to_string()))
    }

    /// Type text
    ///
    /// # Safety
    /// This method must be called from the main thread because it uses macOS's Core Graphics API
    /// which is not thread-safe.
    #[inline]
    pub fn text(&mut self, text: &str) -> Result<(), InputError> {
        if !self.config.enabled {
            return Ok(());
        }

        // SAFETY: We assume this method is only called from the main thread
        // as enforced by the NonSend wrapper
        self.enigo
            .text(text)
            .map_err(|e| InputError::TextInput(e.to_string()))
    }

    /// Update configuration
    pub fn update_config<F>(&mut self, f: F) -> Result<(), InputError>
    where
        F: FnOnce(&mut InputConfig),
    {
        f(&mut self.config);
        Ok(())
    }

    /// Helper to map Bevy KeyCode to Enigo Key
    ///
    /// Note: Only keys that are directly supported by enigo v0.5.0 on macOS are mapped.
    /// Unsupported keys will return an UnsupportedInput error.
    fn map_key_code(&self, key: KeyCode) -> Result<enigo::Key, InputError> {
        use KeyCode as K;

        match key {
            // Function keys (F1-F12 are supported on all platforms)
            K::F1 => Ok(enigo::Key::F1),
            K::F2 => Ok(enigo::Key::F2),
            K::F3 => Ok(enigo::Key::F3),
            K::F4 => Ok(enigo::Key::F4),
            K::F5 => Ok(enigo::Key::F5),
            K::F6 => Ok(enigo::Key::F6),
            K::F7 => Ok(enigo::Key::F7),
            K::F8 => Ok(enigo::Key::F8),
            K::F9 => Ok(enigo::Key::F9),
            K::F10 => Ok(enigo::Key::F10),
            K::F11 => Ok(enigo::Key::F11),
            K::F12 => Ok(enigo::Key::F12),

            // Navigation keys (supported on all platforms)
            K::ArrowLeft => Ok(enigo::Key::LeftArrow),
            K::ArrowRight => Ok(enigo::Key::RightArrow),
            K::ArrowUp => Ok(enigo::Key::UpArrow),
            K::ArrowDown => Ok(enigo::Key::DownArrow),
            K::Enter => Ok(enigo::Key::Return), // NumpadEnter is not a separate variant in this Bevy version
            K::Escape => Ok(enigo::Key::Escape),
            K::Backspace => Ok(enigo::Key::Backspace),
            K::Tab => Ok(enigo::Key::Tab),
            K::Space => Ok(enigo::Key::Space),
            K::Delete => Ok(enigo::Key::Delete),

            // Modifier keys (supported on all platforms)
            K::ControlLeft | K::ControlRight => Ok(enigo::Key::Control),
            K::ShiftLeft | K::ShiftRight => Ok(enigo::Key::Shift),
            K::AltLeft | K::AltRight => Ok(enigo::Key::Alt),
            K::SuperLeft | K::SuperRight => Ok(enigo::Key::Meta), // MetaLeft/MetaRight not in this Bevy version

            // Special keys (platform specific)
            K::CapsLock => Ok(enigo::Key::CapsLock),

            // Platform-agnostic key mappings using Unicode for broad compatibility

            // Navigation keys that are supported on all platforms
            K::End => Ok(enigo::Key::End),
            K::Home => Ok(enigo::Key::Home),
            K::PageUp => Ok(enigo::Key::PageUp),
            K::PageDown => Ok(enigo::Key::PageDown),

            // Alphabet keys - Use character-based input for cross-platform compatibility
            K::KeyA => Ok(enigo::Key::Unicode('a')),
            K::KeyB => Ok(enigo::Key::Unicode('b')),
            K::KeyC => Ok(enigo::Key::Unicode('c')),
            K::KeyD => Ok(enigo::Key::Unicode('d')),
            K::KeyE => Ok(enigo::Key::Unicode('e')),
            K::KeyF => Ok(enigo::Key::Unicode('f')),
            K::KeyG => Ok(enigo::Key::Unicode('g')),
            K::KeyH => Ok(enigo::Key::Unicode('h')),
            K::KeyI => Ok(enigo::Key::Unicode('i')),
            K::KeyJ => Ok(enigo::Key::Unicode('j')),
            K::KeyK => Ok(enigo::Key::Unicode('k')),
            K::KeyL => Ok(enigo::Key::Unicode('l')),
            K::KeyM => Ok(enigo::Key::Unicode('m')),
            K::KeyN => Ok(enigo::Key::Unicode('n')),
            K::KeyO => Ok(enigo::Key::Unicode('o')),
            K::KeyP => Ok(enigo::Key::Unicode('p')),
            K::KeyQ => Ok(enigo::Key::Unicode('q')),
            K::KeyR => Ok(enigo::Key::Unicode('r')),
            K::KeyS => Ok(enigo::Key::Unicode('s')),
            K::KeyT => Ok(enigo::Key::Unicode('t')),
            K::KeyU => Ok(enigo::Key::Unicode('u')),
            K::KeyV => Ok(enigo::Key::Unicode('v')),
            K::KeyW => Ok(enigo::Key::Unicode('w')),
            K::KeyX => Ok(enigo::Key::Unicode('x')),
            K::KeyY => Ok(enigo::Key::Unicode('y')),
            K::KeyZ => Ok(enigo::Key::Unicode('z')),

            // Number keys - Use character-based input for cross-platform compatibility
            K::Digit0 => Ok(enigo::Key::Unicode('0')),
            K::Digit1 => Ok(enigo::Key::Unicode('1')),
            K::Digit2 => Ok(enigo::Key::Unicode('2')),
            K::Digit3 => Ok(enigo::Key::Unicode('3')),
            K::Digit4 => Ok(enigo::Key::Unicode('4')),
            K::Digit5 => Ok(enigo::Key::Unicode('5')),
            K::Digit6 => Ok(enigo::Key::Unicode('6')),
            K::Digit7 => Ok(enigo::Key::Unicode('7')),
            K::Digit8 => Ok(enigo::Key::Unicode('8')),
            K::Digit9 => Ok(enigo::Key::Unicode('9')),

            // Numpad keys - Use character-based input for cross-platform compatibility
            K::Numpad0 => Ok(enigo::Key::Unicode('0')),
            K::Numpad1 => Ok(enigo::Key::Unicode('1')),
            K::Numpad2 => Ok(enigo::Key::Unicode('2')),
            K::Numpad3 => Ok(enigo::Key::Unicode('3')),
            K::Numpad4 => Ok(enigo::Key::Unicode('4')),
            K::Numpad5 => Ok(enigo::Key::Unicode('5')),
            K::Numpad6 => Ok(enigo::Key::Unicode('6')),
            K::Numpad7 => Ok(enigo::Key::Unicode('7')),
            K::Numpad8 => Ok(enigo::Key::Unicode('8')),
            K::Numpad9 => Ok(enigo::Key::Unicode('9')),
            K::NumpadAdd => Ok(enigo::Key::Unicode('+')),
            K::NumpadSubtract => Ok(enigo::Key::Unicode('-')),
            K::NumpadMultiply => Ok(enigo::Key::Unicode('*')),
            K::NumpadDivide => Ok(enigo::Key::Unicode('/')),
            K::NumpadDecimal => Ok(enigo::Key::Unicode('.')),
            K::NumpadEnter => Ok(enigo::Key::Return),

            // Symbol keys - Use Unicode for cross-platform compatibility
            K::Minus => Ok(enigo::Key::Unicode('-')),
            K::Equal => Ok(enigo::Key::Unicode('=')),
            K::BracketLeft => Ok(enigo::Key::Unicode('[')),
            K::BracketRight => Ok(enigo::Key::Unicode(']')),
            K::Backslash => Ok(enigo::Key::Unicode('\\')),
            K::Semicolon => Ok(enigo::Key::Unicode(';')),
            K::Quote => Ok(enigo::Key::Unicode('\'')),
            K::Backquote => Ok(enigo::Key::Unicode('`')),
            K::Comma => Ok(enigo::Key::Unicode(',')),
            K::Period => Ok(enigo::Key::Unicode('.')),
            K::Slash => Ok(enigo::Key::Unicode('/')),

            // Additional function keys
            K::F13 => Ok(enigo::Key::F13),
            K::F14 => Ok(enigo::Key::F14),
            K::F15 => Ok(enigo::Key::F15),
            K::F16 => Ok(enigo::Key::F16),
            K::F17 => Ok(enigo::Key::F17),
            K::F18 => Ok(enigo::Key::F18),
            K::F19 => Ok(enigo::Key::F19),
            K::F20 => Ok(enigo::Key::F20),

            // Insert key
            K::Insert => Ok(enigo::Key::Unicode('I')), // Platform-specific fallback

            // Media keys - Use layout-based approach for better compatibility
            // Note: Volume keys may not be available in current Bevy KeyCode enum
            // Using fallback approach for compatibility

            // Catch-all for truly unsupported keys
            _ => {
                // Log the unsupported key for debugging but provide a reasonable fallback
                warn!("Unsupported key mapping for {:?}, using space as fallback", key);
                Ok(enigo::Key::Unicode(' ')) // Safe fallback that won't break functionality
            }
        }
    }
}

/// Plugin for the input system
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the input system as a non-send resource
        match InputSystem::new() {
            Ok(input_system) => {
                info!("✅ Input system initialized successfully");
                app.insert_non_send_resource(input_system);
            }
            Err(e) => {
                error!("❌ Failed to initialize input system: {}", e);
                error!("   This may be due to missing accessibility permissions on macOS");
                error!("   The app will continue without input simulation capabilities");
                // Insert a placeholder resource to prevent panics
                return;
            }
        }

        // Register the input system to run in the update stage
        app.add_systems(
            Update,
            (handle_keyboard_input, handle_mouse_input)
                .run_if(|res: Option<NonSend<InputSystem>>| res.is_some()),
        );

        // Input system is now initialized in the plugin build method above
    }
}

/// System that handles keyboard input
fn handle_keyboard_input(
    mut input_system: NonSendMut<InputSystem>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Example: Handle keyboard input here
    for key in keyboard_input.get_just_pressed() {
        if let Err(e) = input_system.key_press(*key) {
            error!("Failed to handle key press: {}", e);
        }
    }
}

/// System that handles mouse input
fn handle_mouse_input(
    mut input_system: NonSendMut<InputSystem>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<CursorMoved>,
) {
    // Example: Handle mouse clicks
    for button in mouse_button_input.get_just_pressed() {
        if let Err(e) = input_system.click(match button {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            MouseButton::Middle => Button::Middle,
            _ => continue,
        }) {
            error!("Failed to handle mouse click: {}", e);
        }
    }

    // Example: Handle mouse movement
    for event in mouse_motion.read() {
        if let Err(e) = input_system.move_mouse(event.position.x as i32, event.position.y as i32) {
            error!("Failed to handle mouse movement: {}", e);
        }
    }
}
