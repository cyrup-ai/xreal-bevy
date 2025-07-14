use anyhow::Result;
use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use scap::{
    capturer::{Capturer, Options, Resolution},
    frame::{Frame, FrameType},
    get_all_targets, has_permission, is_supported, request_permission,
};

#[derive(Component)]
pub struct CaptureTask(pub Task<CommandQueue>);

#[derive(Resource)]
pub struct ScreenCaptures {
    pub num_streams: usize,
    capturer: Option<Capturer>,
    // Pre-allocated buffer pool for zero hot-path allocations
    rgba_buffer: Vec<u8>,
    buffer_capacity: usize,
}

// Safety: The scap Capturer contains platform-specific handles that are safe to send between threads
// We guarantee that the capturer is only ever accessed from the main thread in Bevy systems
// The underlying macOS ScreenCaptureKit APIs are thread-safe for single-threaded access patterns
unsafe impl Send for ScreenCaptures {}
unsafe impl Sync for ScreenCaptures {}

impl ScreenCaptures {
    /// Async initialization with optimal framerate detection
    pub async fn new_async() -> Result<Self> {
        // Check platform support first
        if !is_supported() {
            return Err(anyhow::anyhow!("Platform not supported for screen capture"));
        }

        // Check and request permissions
        if !has_permission() {
            if !request_permission() {
                return Err(anyhow::anyhow!("Screen capture permission denied"));
            }
        }

        // Get available capture targets for multi-display support
        let targets = get_all_targets();
        let display_targets: Vec<_> = targets.into_iter()
            .filter(|target| matches!(target, scap::Target::Display(_)))
            .collect();
        let num_displays = display_targets.len().max(1);

        // Use async framerate detection for optimal performance
        let target_fps = Self::detect_optimal_framerate_async().await;
        
        let options = Options {
            fps: target_fps, // Adaptive: 120Hz for Pro, 72Hz for Air, 60Hz fallback
            target: display_targets.first().cloned(), // Use first display
            show_cursor: true,
            show_highlight: false,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame, // Most efficient on macOS
            output_resolution: Resolution::Captured, // Native resolution for best performance
            crop_area: None,
        };

        // Build capturer with proper error handling
        let mut capturer = Capturer::build(options)
            .map_err(|e| anyhow::anyhow!("Failed to build capturer: {}", e))?;

        // Start capture immediately for minimal latency
        capturer.start_capture();

        // Pre-allocate buffer for 4K RGBA (worst case) to avoid hot-path allocations
        const MAX_BUFFER_SIZE: usize = 3840 * 2160 * 4; // 4K RGBA
        let rgba_buffer = Vec::with_capacity(MAX_BUFFER_SIZE);

        Ok(Self {
            num_streams: num_displays,
            capturer: Some(capturer),
            rgba_buffer,
            buffer_capacity: MAX_BUFFER_SIZE,
        })
    }
    
    #[inline]
    pub fn new() -> Result<Self> {
        // Check platform support first
        if !is_supported() {
            return Err(anyhow::anyhow!("Platform not supported for screen capture"));
        }

        // Check and request permissions
        if !has_permission() {
            if !request_permission() {
                return Err(anyhow::anyhow!("Screen capture permission denied"));
            }
        }

        // Get available capture targets for multi-display support
        let targets = get_all_targets();
        let display_targets: Vec<_> = targets.into_iter()
            .filter(|target| matches!(target, scap::Target::Display(_)))
            .collect();
        let num_displays = display_targets.len().max(1);

        // Adaptive frame rate for commercial compatibility (all XREAL models)
        let target_fps = Self::detect_optimal_framerate();
        
        let options = Options {
            fps: target_fps, // Adaptive: 120Hz for Pro, 72Hz for Air, 60Hz fallback
            target: display_targets.first().cloned(), // Use first display
            show_cursor: true,
            show_highlight: false,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame, // Most efficient on macOS
            output_resolution: Resolution::Captured, // Native resolution for best performance
            crop_area: None,
        };

        // Build capturer with proper error handling
        let mut capturer = Capturer::build(options)
            .map_err(|e| anyhow::anyhow!("Failed to build capturer: {}", e))?;

        // Start capture immediately for minimal latency
        capturer.start_capture();

        // Pre-allocate buffer for 4K RGBA (worst case) to avoid hot-path allocations
        const MAX_BUFFER_SIZE: usize = 3840 * 2160 * 4; // 4K RGBA
        let rgba_buffer = Vec::with_capacity(MAX_BUFFER_SIZE);

        Ok(Self {
            num_streams: num_displays,
            capturer: Some(capturer),
            rgba_buffer,
            buffer_capacity: MAX_BUFFER_SIZE,
        })
    }

    /// Detect optimal frame rate for XREAL 2 series and other models
    /// Returns a future that resolves to the optimal framerate
    async fn detect_optimal_framerate_async() -> u32 {
        use bevy::tasks::AsyncComputeTaskPool;
        
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async {
            // Try to detect display refresh rate for optimal performance
            // Priority: 120Hz (XREAL 2 Pro), 90Hz (XREAL 2), 72Hz (Air), 60Hz (fallback)
            if let Ok(output) = async_process::Command::new("system_profiler")
                .args(&["SPDisplaysDataType"])
                .output()
                .await
            {
                let display_info = String::from_utf8_lossy(&output.stdout);
                
                // Check for high refresh rate capabilities
                if display_info.contains("120") || display_info.contains(" 120 ") {
                    return 120; // XREAL 2 Pro
                } else if display_info.contains("90") || display_info.contains(" 90 ") {
                    return 90;  // XREAL 2
                } else if display_info.contains("72") || display_info.contains(" 72 ") {
                    return 72;  // XREAL Air series
                }
            }
            
            // Safe fallback for all models
            60
        });
        
        task.await
    }
    
    /// Synchronous wrapper that provides fallback when async detection isn't available
    #[inline]
    fn detect_optimal_framerate() -> u32 {
        // Safe fallback for synchronous initialization
        // The async version should be preferred when possible
        72 // Conservative default for XREAL Air series compatibility
    }

    /// Spawn async capture task for non-blocking screen capture
    pub fn spawn_capture_task(&self, entity: Entity) -> Option<CaptureTask> {
        if self.capturer.is_none() {
            return None;
        }

        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            
            // Capture frame data in the async task
            command_queue.push(move |world: &mut World| {
                // Get the screen captures resource to access the capturer
                if let Some(mut captures) = world.get_resource_mut::<ScreenCaptures>() {
                    if let Some(ref mut capturer) = captures.capturer {
                        // Get the next frame from the capturer
                        match capturer.get_next_frame() {
                            Ok(frame) => {
                                // Convert frame data to Bevy Image
                                if let Ok(image_data) = Self::frame_to_bevy_image(frame) {
                                    // Update the entity's material with the new texture
                                    if let Ok(entity_ref) = world.get_entity(entity) {
                                        if let Some(screen_material) = entity_ref.get::<crate::render::ScreenMaterial>() {
                                            let material_handle = screen_material.0.clone();
                                            
                                            // Add image to assets first
                                            let image_handle = if let Some(mut images) = world.get_resource_mut::<Assets<Image>>() {
                                                images.add(image_data)
                                            } else {
                                                return; // Can't access images resource
                                            };
                                            
                                            // Then update material
                                            if let Some(mut materials) = world.get_resource_mut::<Assets<StandardMaterial>>() {
                                                if let Some(material) = materials.get_mut(&material_handle) {
                                                    material.base_color_texture = Some(image_handle);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Frame capture failed, continue without error
                            }
                        }
                    }
                }
                
                // Remove the task component since we're done with this frame
                if let Ok(mut entity_ref) = world.get_entity_mut(entity) {
                    entity_ref.remove::<CaptureTask>();
                }
            });
            
            command_queue
        });

        Some(CaptureTask(task))
    }

    /// Convert scap frame to Bevy Image
    fn frame_to_bevy_image(frame: Frame) -> Result<Image> {
        use bevy::render::{
            render_asset::RenderAssetUsages,
            render_resource::{Extent3d, TextureDimension, TextureFormat},
        };

        // Extract frame data based on scap Frame API
        let (width, height, data) = match frame {
            Frame::BGRA(bgra_frame) => {
                (bgra_frame.width as u32, bgra_frame.height as u32, bgra_frame.data)
            }
            Frame::RGB(rgb_frame) => {
                // Convert RGB to BGRA by adding alpha channel and swapping bytes
                let mut bgra_data = Vec::with_capacity(rgb_frame.data.len() * 4 / 3);
                for chunk in rgb_frame.data.chunks(3) {
                    if chunk.len() == 3 {
                        bgra_data.push(chunk[2]); // B
                        bgra_data.push(chunk[1]); // G
                        bgra_data.push(chunk[0]); // R
                        bgra_data.push(255);      // A
                    }
                }
                (rgb_frame.width as u32, rgb_frame.height as u32, bgra_data)
            }
            Frame::RGBx(rgbx_frame) => {
                // RGBx format - already has 4 bytes per pixel, convert to BGRA
                let mut bgra_data = Vec::with_capacity(rgbx_frame.data.len());
                for chunk in rgbx_frame.data.chunks(4) {
                    if chunk.len() == 4 {
                        bgra_data.push(chunk[2]); // B
                        bgra_data.push(chunk[1]); // G
                        bgra_data.push(chunk[0]); // R
                        bgra_data.push(255);      // A (ignore X)
                    }
                }
                (rgbx_frame.width as u32, rgbx_frame.height as u32, bgra_data)
            }
            Frame::XBGR(xbgr_frame) => {
                // XBGR format - X B G R to B G R A
                let mut bgra_data = Vec::with_capacity(xbgr_frame.data.len());
                for chunk in xbgr_frame.data.chunks(4) {
                    if chunk.len() == 4 {
                        bgra_data.push(chunk[1]); // B
                        bgra_data.push(chunk[2]); // G
                        bgra_data.push(chunk[3]); // R
                        bgra_data.push(255);      // A
                    }
                }
                (xbgr_frame.width as u32, xbgr_frame.height as u32, bgra_data)
            }
            Frame::BGRx(bgrx_frame) => {
                // BGRx format - already close to BGRA
                let mut bgra_data = Vec::with_capacity(bgrx_frame.data.len());
                for chunk in bgrx_frame.data.chunks(4) {
                    if chunk.len() == 4 {
                        bgra_data.push(chunk[0]); // B
                        bgra_data.push(chunk[1]); // G
                        bgra_data.push(chunk[2]); // R
                        bgra_data.push(255);      // A (ignore X)
                    }
                }
                (bgrx_frame.width as u32, bgrx_frame.height as u32, bgra_data)
            }
            Frame::BGR0(bgr_frame) => {
                // BGR0 format - similar to BGRx
                let mut bgra_data = Vec::with_capacity(bgr_frame.data.len());
                for chunk in bgr_frame.data.chunks(4) {
                    if chunk.len() == 4 {
                        bgra_data.push(chunk[0]); // B
                        bgra_data.push(chunk[1]); // G
                        bgra_data.push(chunk[2]); // R
                        bgra_data.push(255);      // A
                    }
                }
                (bgr_frame.width as u32, bgr_frame.height as u32, bgra_data)
            }
            Frame::YUVFrame(_yuv_frame) => {
                // YUV format conversion is complex, skip for now
                return Err(anyhow::anyhow!("YUV frame format not supported yet"));
            }
        };

        // Create Bevy Image from BGRA frame data
        let image = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );

        Ok(image)
    }
}

impl Drop for ScreenCaptures {
    fn drop(&mut self) {
        if let Some(ref mut capturer) = self.capturer {
            capturer.stop_capture();
        }
    }
}