#[expect(clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use anyhow::anyhow;

use ctb_storage_minimal::xkb::ensure_xkb_config_root;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct KeyboardMap {
    min_keycode: u8,
    keysyms_per_keycode: u8,
    keysyms: Vec<yaxi::keyboard::Keysym>,
}

#[derive(Clone, Copy)]
struct XidCursor {
    next: u32,
    increment: u32,
    mask: u32,
    exhausted: bool,
}

impl XidCursor {
    fn new(mask: u32) -> Result<Self> {
        let increment = mask & mask.wrapping_neg();
        ensure!(mask != 0 && increment != 0, "X11 display ran out of XIDs");

        Ok(Self {
            next: 0,
            increment,
            mask,
            exhausted: false,
        })
    }

    fn reserve(&mut self, allocations: usize) -> Result<u32> {
        ensure!(!self.exhausted, "X11 display ran out of XIDs");
        ensure!(allocations > 0, "X11 XID reservations must be non-zero");

        let seed = self.next;

        for _ in 0..allocations {
            let Some(candidate) = self.next.checked_add(self.increment) else {
                self.exhausted = true;
                return Ok(seed);
            };

            if candidate & !self.mask != 0 {
                self.exhausted = true;
                return Ok(seed);
            }

            self.next = candidate;
        }

        Ok(seed)
    }
}

/// Describes the subset of the X11 client surface this crate owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X11Capability {
    Clipboard,
    EventLoop,
    InputMethod,
    SurfacePresentation,
    WindowCreation,
}

impl fmt::Display for X11Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Clipboard => "clipboard",
            Self::EventLoop => "event loop",
            Self::InputMethod => "input method",
            Self::SurfacePresentation => "surface presentation",
            Self::WindowCreation => "window creation",
        };

        f.write_str(name)
    }
}

/// Parameters for creating a simple top-level X11 window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowOptions {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

impl WindowOptions {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
            border_width: 0,
        }
    }
}

/// Basic pixel surface metadata needed by software-rendered backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceInfo {
    pub width: u16,
    pub height: u16,
    pub depth: u8,
    pub visual_id: u32,
}

/// Thin wrapper around a live X11 display connection.
#[derive(Clone)]
pub struct DisplayHandle {
    inner: yaxi::display::Display,
    keyboard_map: Arc<Mutex<Option<KeyboardMap>>>,
    xid_cursor: Arc<Mutex<XidCursor>>,
}

impl DisplayHandle {
    fn prepare_xid_allocations(&self, allocations: usize) -> Result<()> {
        let mut xid_cursor = self.xid_cursor.lock().map_err(|lock_error| {
            anyhow!("failed to lock X11 XID cursor: {lock_error}")
        })?;
        let next = xid_cursor.reserve(allocations)?;

        self.inner
            .refresh_xid_allocator_at(next)
            .map_err(|error| anyhow!(error))
    }

    fn populate_keyboard_map(&self) -> Result<()> {
        let mut keymap_guard =
            self.keyboard_map.lock().map_err(|lock_error| {
                anyhow!("failed to lock X11 keyboard map cache: {lock_error}")
            })?;

        if keymap_guard.is_none() {
            let range = self.inner.display_keycodes();
            let (keysyms, keysyms_per_keycode) = self
                .inner
                .get_keyboard_mapping()
                .map_err(|error| anyhow!(error))?;

            *keymap_guard = Some(KeyboardMap {
                min_keycode: range.min,
                keysyms_per_keycode,
                keysyms,
            });
        }

        Ok(())
    }

    /// Opens an X11 display and ensures bundled XKB data is available first.
    pub fn open(display_name: Option<&str>) -> Result<Self> {
        ensure_xkb_config_root().context("prepare XKB config for X11")?;

        let inner = yaxi::display::open(display_name)
            .map_err(|error| anyhow!(error))?;
        let xid_cursor = XidCursor::new(inner.xid_mask())?;

        let handle = Self {
            inner,
            keyboard_map: Arc::new(Mutex::new(None)),
            xid_cursor: Arc::new(Mutex::new(xid_cursor)),
        };
        handle.populate_keyboard_map()?;

        Ok(handle)
    }

    /// Returns true when an event is ready to be consumed.
    pub fn poll_event(&self) -> Result<bool> {
        self.inner.poll_event().map_err(|error| anyhow!(error))
    }

    pub fn next_event(&self) -> Result<yaxi::proto::Event> {
        self.inner.next_event().map_err(|error| anyhow!(error))
    }

    pub fn intern_atom(
        &self,
        name: &str,
        only_if_exists: bool,
    ) -> Result<yaxi::display::Atom> {
        self.inner
            .intern_atom(name, only_if_exists)
            .map_err(|error| anyhow!(error))
    }

    pub fn keysym_from_keycode(
        &self,
        keycode: u8,
    ) -> Result<yaxi::keyboard::Keysym> {
        self.populate_keyboard_map()?;

        let keymap_guard = self.keyboard_map.lock().map_err(|lock_error| {
            anyhow!("failed to lock X11 keyboard map cache: {lock_error}")
        })?;

        let Some(keymap) = keymap_guard.as_ref() else {
            bail!("X11 keyboard map cache was not initialized");
        };

        ensure!(
            keycode >= keymap.min_keycode,
            "X11 keycode {} below minimum {}",
            keycode,
            keymap.min_keycode
        );

        let offset = usize::from(keycode.saturating_sub(keymap.min_keycode))
            .saturating_mul(usize::from(keymap.keysyms_per_keycode));
        keymap.keysyms.get(offset).copied().ok_or_else(|| {
            anyhow!("X11 keycode {keycode} missing from cached keymap")
        })
    }

    pub fn invalidate_keyboard_map(&self) -> Result<()> {
        let mut keymap_guard =
            self.keyboard_map.lock().map_err(|lock_error| {
                anyhow!("failed to lock X11 keyboard map cache: {lock_error}")
            })?;
        *keymap_guard = None;
        Ok(())
    }

    /// Opens a dedicated clipboard helper on the same display selection.
    pub fn clipboard(
        &self,
        display_name: Option<&str>,
    ) -> Result<ClipboardHandle> {
        ClipboardHandle::open(display_name)
    }

    /// Creates a simple mapped-window placeholder suitable for later software surface work.
    pub fn create_window(
        &self,
        options: WindowOptions,
    ) -> Result<WindowHandle> {
        self.prepare_xid_allocations(1)?;

        let root = self
            .inner
            .default_root_window()
            .map_err(|error| anyhow!(error))?;

        let window = root
            .create_window(yaxi::window::WindowArguments {
                depth: root.depth(),
                x: options.x,
                y: options.y,
                width: options.width,
                height: options.height,
                border_width: options.border_width,
                class: yaxi::proto::WindowClass::InputOutput,
                visual: root.visual(),
                values: yaxi::window::ValuesBuilder::new(vec![]),
            })
            .map_err(|error| anyhow!(error))?;

        Ok(WindowHandle {
            surface: SurfaceInfo {
                width: options.width,
                height: options.height,
                depth: window.depth(),
                visual_id: window.visual().id,
            },
            inner: window,
            display: self.clone(),
        })
    }
}

/// Thin wrapper around yaxi's X11 clipboard implementation.
pub struct ClipboardHandle {
    inner: yaxi::clipboard::Clipboard,
}

impl ClipboardHandle {
    pub fn open(display_name: Option<&str>) -> Result<Self> {
        let inner = yaxi::clipboard::Clipboard::new(display_name)
            .map_err(|error| anyhow!(error))?;

        Ok(Self { inner })
    }

    pub fn read_text(&self) -> Result<Option<String>> {
        self.inner.get_text().map_err(|error| anyhow!(error))
    }

    pub fn write_text(&mut self, value: impl Into<String>) -> Result<()> {
        self.inner
            .set_text(&value.into())
            .map_err(|error| anyhow!(error))
    }

    pub fn read_primary_text(&self) -> Result<Option<String>> {
        self.inner
            .get_primary_text()
            .map_err(|error| anyhow!(error))
    }

    pub fn write_primary_text(
        &mut self,
        value: impl Into<String>,
    ) -> Result<()> {
        self.inner
            .set_primary_text(&value.into())
            .map_err(|error| anyhow!(error))
    }
}

/// Handle for a simple top-level X11 window plus software-surface metadata.
pub struct WindowHandle {
    inner: yaxi::window::Window,
    surface: SurfaceInfo,
    display: DisplayHandle,
}

impl WindowHandle {
    pub fn map(&self) -> Result<()> {
        self.inner
            .map(yaxi::window::WindowKind::Window)
            .map_err(|error| anyhow!(error))
    }

    pub fn destroy(self) -> Result<()> {
        self.inner
            .destroy(yaxi::window::WindowKind::Window)
            .map_err(|error| anyhow!(error))
    }

    pub fn surface_info(&self) -> SurfaceInfo {
        self.surface
    }

    pub fn select_input(&self, masks: &[yaxi::proto::EventMask]) -> Result<()> {
        self.inner
            .select_input(masks)
            .map_err(|error| anyhow!(error))
    }

    pub fn set_title(
        &self,
        display: &DisplayHandle,
        title: &str,
    ) -> Result<()> {
        let net_wm_name = display.intern_atom("_NET_WM_NAME", false)?;
        let utf8 = display.intern_atom("UTF8_STRING", false)?;

        self.inner
            .change_property(
                net_wm_name,
                utf8,
                yaxi::window::PropFormat::Format8,
                yaxi::window::PropMode::Replace,
                title.as_bytes(),
            )
            .map_err(|error| anyhow!(error))
    }

    pub fn enable_wm_delete(
        &self,
        display: &DisplayHandle,
    ) -> Result<yaxi::display::Atom> {
        let wm_protocols = display.intern_atom("WM_PROTOCOLS", false)?;
        let wm_delete_window =
            display.intern_atom("WM_DELETE_WINDOW", false)?;

        self.inner
            .change_property(
                wm_protocols,
                yaxi::display::Atom::ATOM,
                yaxi::window::PropFormat::Format32,
                yaxi::window::PropMode::Replace,
                &wm_delete_window.id().to_le_bytes(),
            )
            .map_err(|error| anyhow!(error))?;

        Ok(wm_delete_window)
    }

    pub fn upload_bgra(&self, buffer: &[[u8; 4]]) -> Result<()> {
        let row_stride = usize::from(self.surface.width).saturating_mul(4);
        ensure!(row_stride > 0, "X11 surface row stride must be non-zero");

        let expected_pixels =
            usize::from(self.surface.width).saturating_mul(usize::from(self.surface.height));
        ensure!(
            buffer.len() == expected_pixels,
            "X11 surface buffer length {} does not match {}x{}",
            buffer.len(),
            self.surface.width,
            self.surface.height
        );

        let max_request_words = usize::from(u16::MAX);
        let max_payload_bytes = max_request_words.saturating_sub(6).saturating_mul(4);
        let rows_per_chunk = std::cmp::max(1, max_payload_bytes.checked_div(row_stride).unwrap_or(1));
        let chunk_count =
            usize::from(self.surface.height).div_ceil(rows_per_chunk);
        self.display.prepare_xid_allocations(chunk_count)?;

        let mut start_row = 0usize;
        while start_row < usize::from(self.surface.height) {
            let end_row = std::cmp::min(
                start_row.saturating_add(rows_per_chunk),
                usize::from(self.surface.height),
            );
            let chunk_height = end_row.saturating_sub(start_row);
            let chunk_start = start_row.saturating_mul(usize::from(self.surface.width));
            let chunk_end = end_row.saturating_mul(usize::from(self.surface.width));
            let chunk = buffer.get(chunk_start..chunk_end).ok_or_else(|| {
                anyhow!(
                    "X11 surface chunk {}..{} is out of bounds for {} pixels",
                    chunk_start,
                    chunk_end,
                    buffer.len()
                )
            })?;

            let mut bytes = Vec::with_capacity(chunk.len().saturating_mul(4));
            for pixel in chunk {
                bytes.extend_from_slice(pixel);
            }

            self.inner
                .put_image(
                    yaxi::window::ImageFormat::ZPixmap,
                    self.surface.width,
                    u16::try_from(chunk_height)
                        .context("convert X11 chunk height to u16")?,
                    0,
                    i16::try_from(start_row)
                        .context("convert X11 chunk Y offset to i16")?,
                    &bytes,
                )
                .map_err(|error| anyhow!(error))?;

            start_row = end_row;
        }

        Ok(())
    }

    pub fn resize_surface(&mut self, width: u16, height: u16) {
        self.surface.width = width;
        self.surface.height = height;
    }

    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Explicit stub until a software-present path is wired through this crate.
    pub fn require_surface_presentation(&self) -> Result<()> {
        unsupported(X11Capability::SurfacePresentation)
    }

    /// Explicit stub until IME/XIM is wired through this crate.
    pub fn require_input_method(&self) -> Result<()> {
        unsupported(X11Capability::InputMethod)
    }
}

pub fn unsupported<T>(capability: X11Capability) -> Result<T> {
    bail!("x11_client {capability} is not wired yet")
}

pub fn rounded_points_to_u16(value: f32) -> Result<u16> {
    let rounded = value.max(1.0).round();
    ensure!(rounded.is_finite(), "non-finite X11 size {rounded}");
    ensure!(
        rounded <= f32::from(u16::MAX),
        "X11 size {rounded} exceeds u16"
    );

    let text = format!("{rounded:.0}");
    text.parse::<u16>()
        .map_err(|error| anyhow!(error))
        .context("convert X11 size to u16")
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_window_options_defaults() {
        let options = WindowOptions::new(640, 480);

        assert_eq!(options.x, 0);
        assert_eq!(options.y, 0);
        assert_eq!(options.width, 640);
        assert_eq!(options.height, 480);
        assert_eq!(options.border_width, 0);
    }

    #[crate::ctb_test]
    fn test_unsupported_error_includes_capability() {
        let error = unsupported::<()>(X11Capability::SurfacePresentation)
            .expect_err("surface presentation should still be stubbed");

        assert!(error.to_string().contains("surface presentation"));
    }
}
