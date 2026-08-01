#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_workspace_x11_client::{
    ClipboardHandle, DisplayHandle, WindowHandle, WindowOptions,
    rounded_points_to_u16,
};
use egui::{
    Pos2, RawInput, Rect, SafeAreaInsets, ViewportId, ViewportIdMap,
    ViewportInfo,
};
use egui_software_backend::{
    App, BufferMutRef, ColorFieldOrder, EguiSoftwareRender, SoftwareBackend,
    SoftwareBackendAppConfiguration,
};
use std::time::{Duration, Instant};

#[expect(clippy::too_many_lines, reason = "large initialization function")]
pub fn run_app_with_x11_client_backend<T: App>(
    settings: &SoftwareBackendAppConfiguration,
    egui_app_factory: impl FnOnce(egui::Context) -> T,
) -> Result<()> {
    let egui_context = egui::Context::default();
    let mut app = egui_app_factory(egui_context.clone());
    let mut software_backend = SoftwareBackend::new();
    let mut renderer = EguiSoftwareRender::new(ColorFieldOrder::Bgra)
        .with_allow_raster_opt(settings.allow_raster_opt)
        .with_convert_tris_to_rects(settings.convert_tris_to_rects)
        .with_caching(settings.caching);

    let size = settings
        .viewport_builder
        .inner_size
        .unwrap_or(egui::Vec2::new(600.0, 500.0));
    let width = rounded_points_to_u16(size.x)?;
    let height = rounded_points_to_u16(size.y)?;

    let display = DisplayHandle::open(None)?;
    let mut clipboard = display.clipboard(None)?;
    let mut window =
        display.create_window(WindowOptions::new(width, height))?;
    window.select_input(&[
        yaxi::proto::EventMask::ButtonPress,
        yaxi::proto::EventMask::ButtonRelease,
        yaxi::proto::EventMask::PointerMotion,
        yaxi::proto::EventMask::StructureNotify,
        yaxi::proto::EventMask::KeyPress,
        yaxi::proto::EventMask::KeyRelease,
    ])?;

    if let Some(title) = settings.viewport_builder.title.as_deref() {
        let _ = window.set_title(&display, title);
    }
    let wm_delete_window = window.enable_wm_delete(&display)?;
    window.map()?;

    let mut pixels =
        vec![[0u8; 4]; usize::from(width).saturating_mul(usize::from(height))];
    let mut last_pointer_pos = Pos2::ZERO;
    let start_time = Instant::now();
    let frame_interval = Duration::from_millis(16);
    let mut should_quit = false;
    let mut deferred_input_events = Vec::new();
    let mut current_modifiers = egui::Modifiers::default();

    while !should_quit {
        let frame_start = Instant::now();
        let mut input_events = std::mem::take(&mut deferred_input_events);

        while display.poll_event()? {
            let event = display.next_event()?;
            if handle_event(
                &display,
                &mut clipboard,
                &mut window,
                &mut last_pointer_pos,
                &mut current_modifiers,
                wm_delete_window,
                event,
                &mut input_events,
            )? {
                should_quit = true;
                break;
            }
        }

        if should_quit {
            break;
        }

        let raw_input = build_raw_input(
            &window,
            Some(start_time.elapsed().as_secs_f64()),
            current_modifiers,
            input_events,
        );

        let full_output = egui_context.run(raw_input, |ctx| {
            app.update(ctx, &mut software_backend);
        });

        sync_primary_selection(
            &mut clipboard,
            &full_output.platform_output.events,
        )?;
        handle_platform_output_commands(
            &mut clipboard,
            &full_output.platform_output.commands,
        )?;

        let mut close_requested = false;
        if let Some(viewport_output) =
            full_output.viewport_output.get(&ViewportId::ROOT)
        {
            for command in &viewport_output.commands {
                match command {
                    egui::ViewportCommand::Close => close_requested = true,
                    egui::ViewportCommand::Title(title) => {
                        let _ = window.set_title(&display, title);
                    }
                    egui::ViewportCommand::RequestCut => {
                        deferred_input_events.push(egui::Event::Cut);
                    }
                    egui::ViewportCommand::RequestCopy => {
                        deferred_input_events.push(egui::Event::Copy);
                    }
                    egui::ViewportCommand::RequestPaste => {
                        if let Some(content) = clipboard.read_text()? {
                            let content = content.replace("\r\n", "\n");
                            if !content.is_empty() {
                                deferred_input_events
                                    .push(egui::Event::Paste(content));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let clipped_primitives = egui_context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let surface = window.surface_info();
        let expected_pixels = usize::from(surface.width)
            .saturating_mul(usize::from(surface.height));
        if pixels.len() != expected_pixels {
            pixels.resize(expected_pixels, [0, 0, 0, 0]);
        }

        let mut buffer_ref = BufferMutRef::new(
            &mut pixels,
            usize::from(surface.width),
            usize::from(surface.height),
        );
        buffer_ref.data.fill([0, 0, 0, 0]);
        renderer.render(
            &mut buffer_ref,
            &clipped_primitives,
            &full_output.textures_delta,
            full_output.pixels_per_point,
        );
        window.upload_bgra(buffer_ref.data)?;

        if close_requested {
            break;
        }

        if let Some(remaining) =
            frame_interval.checked_sub(frame_start.elapsed())
        {
            std::thread::sleep(remaining);
        }
    }

    app.on_exit(&egui_context);
    window.destroy()?;
    Ok(())
}

fn build_raw_input(
    window: &WindowHandle,
    time: Option<f64>,
    modifiers: egui::Modifiers,
    events: Vec<egui::Event>,
) -> RawInput {
    let surface = window.surface_info();
    let size = egui::vec2(f32::from(surface.width), f32::from(surface.height));
    let screen_rect = Rect::from_min_size(Pos2::ZERO, size);
    let mut viewports = ViewportIdMap::default();
    viewports.insert(
        ViewportId::ROOT,
        ViewportInfo {
            native_pixels_per_point: Some(1.0),
            inner_rect: Some(screen_rect),
            ..ViewportInfo::default()
        },
    );

    RawInput {
        viewport_id: ViewportId::ROOT,
        viewports,
        screen_rect: Some(screen_rect),
        max_texture_side: Some(8192),
        time,
        predicted_dt: 1.0 / 60.0,
        modifiers,
        events,
        hovered_files: Vec::new(),
        dropped_files: Vec::new(),
        focused: true,
        system_theme: None,
        safe_area_insets: Some(SafeAreaInsets::default()),
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "complex event handler function"
)]
fn handle_event(
    display: &DisplayHandle,
    clipboard: &mut ClipboardHandle,
    window: &mut WindowHandle,
    last_pointer_pos: &mut Pos2,
    current_modifiers: &mut egui::Modifiers,
    wm_delete_window: yaxi::display::Atom,
    event: yaxi::proto::Event,
    output: &mut Vec<egui::Event>,
) -> Result<bool> {
    match event {
        // Basic pointer translation follows the same high-level event mapping as egui-winit.
        yaxi::proto::Event::ButtonEvent {
            kind,
            coordinates,
            window: event_window,
            button,
            state,
            ..
        } if event_window == window.id() => match button {
            yaxi::proto::Button::Button1 | yaxi::proto::Button::Button3 => {
                *current_modifiers = modifiers_from_state(state);
                let pressed = kind == yaxi::proto::EventKind::Press;
                let position = Pos2::new(
                    f32::from(coordinates.x),
                    f32::from(coordinates.y),
                );
                *last_pointer_pos = position;
                output.push(egui::Event::PointerMoved(position));
                output.push(egui::Event::PointerButton {
                    pos: position,
                    button: match button {
                        yaxi::proto::Button::Button1 => {
                            egui::PointerButton::Primary
                        }
                        yaxi::proto::Button::Button3 => {
                            egui::PointerButton::Secondary
                        }
                        _ => unreachable!(),
                    },
                    pressed,
                    modifiers: *current_modifiers,
                });
            }
            yaxi::proto::Button::Button2 => {
                *current_modifiers = modifiers_from_state(state);
                let pressed = kind == yaxi::proto::EventKind::Press;
                let position = Pos2::new(
                    f32::from(coordinates.x),
                    f32::from(coordinates.y),
                );
                *last_pointer_pos = position;
                output.push(egui::Event::PointerMoved(position));
                output.push(egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Middle,
                    pressed,
                    modifiers: *current_modifiers,
                });

                if kind == yaxi::proto::EventKind::Release {
                    if let Some(content) = clipboard.read_primary_text()? {
                        let content = content.replace("\r\n", "\n");
                        if !content.is_empty() {
                            output.push(egui::Event::Paste(content));
                        }
                    }
                }
            }
            yaxi::proto::Button::Button4 => {
                *current_modifiers = modifiers_from_state(state);
                if kind == yaxi::proto::EventKind::Press {
                    output.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Line,
                        delta: egui::vec2(0.0, 1.0),
                        modifiers: *current_modifiers,
                    });
                }
            }
            yaxi::proto::Button::Button5 => {
                *current_modifiers = modifiers_from_state(state);
                if kind == yaxi::proto::EventKind::Press {
                    output.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Line,
                        delta: egui::vec2(0.0, -1.0),
                        modifiers: *current_modifiers,
                    });
                }
            }
        },
        yaxi::proto::Event::MotionNotify {
            coordinates,
            window: event_window,
            state,
            ..
        } if event_window == window.id() => {
            *current_modifiers = modifiers_from_state(state);
            let position =
                Pos2::new(f32::from(coordinates.x), f32::from(coordinates.y));
            *last_pointer_pos = position;
            output.push(egui::Event::PointerMoved(position));
        }
        yaxi::proto::Event::KeyEvent {
            kind,
            window: event_window,
            keycode,
            state,
            ..
        } if event_window == window.id() => {
            let pressed = kind == yaxi::proto::EventKind::Press;
            if let Ok(keysym) = display.keysym_from_keycode(keycode) {
                *current_modifiers =
                    modifiers_from_key_event(state, keysym.raw(), pressed);

                if let Some(key) = egui_key_from_keysym(keysym) {
                    if pressed {
                        if is_cut_command(*current_modifiers, key) {
                            output.push(egui::Event::Cut);
                            return Ok(false);
                        }
                        if is_copy_command(*current_modifiers, key) {
                            output.push(egui::Event::Copy);
                            return Ok(false);
                        }
                        if is_paste_command(*current_modifiers, key) {
                            if let Some(content) = clipboard.read_text()? {
                                let content = content.replace("\r\n", "\n");
                                if !content.is_empty() {
                                    output.push(egui::Event::Paste(content));
                                }
                            }
                            return Ok(false);
                        }
                    }

                    output.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: *current_modifiers,
                    });
                }

                if pressed {
                    if let Ok(character) = keysym.character() {
                        let is_command = current_modifiers.ctrl
                            || current_modifiers.command
                            || current_modifiers.mac_cmd;
                        if !is_command && is_printable_char(character) {
                            output
                                .push(egui::Event::Text(character.to_string()));
                        }
                    }
                }
            }
        }
        yaxi::proto::Event::ConfigureNotify {
            window: event_window,
            width,
            height,
            ..
        } if event_window == window.id() => {
            window.resize_surface(width, height);
        }
        yaxi::proto::Event::MappingNotify { .. } => {
            display.invalidate_keyboard_map()?;
        }
        yaxi::proto::Event::DestroyNotify {
            window: event_window,
            ..
        } if event_window == window.id() => {
            return Ok(true);
        }
        yaxi::proto::Event::ClientMessage {
            window: event_window,
            type_,
            data: yaxi::proto::ClientMessageData::Long(data),
            ..
        } if event_window == window.id() => {
            let wm_protocols = display.intern_atom("WM_PROTOCOLS", false)?;
            if type_ == wm_protocols && data[0] == wm_delete_window.id() {
                return Ok(true);
            }
        }
        _ => {}
    }

    Ok(false)
}

fn handle_platform_output_commands(
    clipboard: &mut ClipboardHandle,
    commands: &[egui::OutputCommand],
) -> Result<()> {
    for command in commands {
        match command {
            egui::OutputCommand::CopyText(text) => {
                clipboard.write_text(text.clone())?;
            }
            egui::OutputCommand::CopyImage(_)
            | egui::OutputCommand::OpenUrl(_) => {}
        }
    }

    Ok(())
}

fn sync_primary_selection(
    clipboard: &mut ClipboardHandle,
    events: &[egui::output::OutputEvent],
) -> Result<()> {
    for event in events {
        let info = event.widget_info();
        let Some(text) = primary_selection_text(info) else {
            continue;
        };
        clipboard.write_primary_text(text)?;
    }

    Ok(())
}

fn primary_selection_text(info: &egui::WidgetInfo) -> Option<String> {
    let text = info.current_text_value.as_ref()?;
    let selection = info.text_selection.as_ref()?;
    let start = (*selection.start()).min(*selection.end());
    let end = (*selection.start()).max(*selection.end());
    if start == end {
        return None;
    }

    Some(
        text.chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect(),
    )
}

fn modifiers_from_state(state: u16) -> egui::Modifiers {
    const SHIFT_MASK: u16 = 0x0001;
    const CONTROL_MASK: u16 = 0x0004;
    const MOD1_MASK: u16 = 0x0008;

    let shift = state & SHIFT_MASK != 0;
    let ctrl = state & CONTROL_MASK != 0;
    let alt = state & MOD1_MASK != 0;

    egui::Modifiers {
        alt,
        ctrl,
        shift,
        mac_cmd: false,
        command: ctrl,
    }
}

fn modifiers_from_key_event(
    state: u16,
    raw_keysym: u32,
    pressed: bool,
) -> egui::Modifiers {
    const XK_SHIFT_L: u32 = 0xffe1;
    const XK_SHIFT_R: u32 = 0xffe2;
    const XK_CONTROL_L: u32 = 0xffe3;
    const XK_CONTROL_R: u32 = 0xffe4;
    const XK_ALT_L: u32 = 0xffe9;
    const XK_ALT_R: u32 = 0xffea;
    const XK_META_L: u32 = 0xffe7;
    const XK_META_R: u32 = 0xffe8;

    let mut modifiers = modifiers_from_state(state);
    match raw_keysym {
        XK_SHIFT_L | XK_SHIFT_R => modifiers.shift = pressed,
        XK_CONTROL_L | XK_CONTROL_R => {
            modifiers.ctrl = pressed;
            modifiers.command = pressed;
        }
        XK_ALT_L | XK_ALT_R | XK_META_L | XK_META_R => modifiers.alt = pressed,
        _ => {}
    }
    modifiers
}

fn egui_key_from_keysym(keysym: yaxi::keyboard::Keysym) -> Option<egui::Key> {
    const XK_BACKSPACE: u32 = 0xff08;
    const XK_TAB: u32 = 0xff09;
    const XK_RETURN: u32 = 0xff0d;
    const XK_ESCAPE: u32 = 0xff1b;
    const XK_HOME: u32 = 0xff50;
    const XK_LEFT: u32 = 0xff51;
    const XK_UP: u32 = 0xff52;
    const XK_RIGHT: u32 = 0xff53;
    const XK_DOWN: u32 = 0xff54;
    const XK_PAGE_UP: u32 = 0xff55;
    const XK_PAGE_DOWN: u32 = 0xff56;
    const XK_END: u32 = 0xff57;
    const XK_DELETE: u32 = 0xffff;
    const XK_INSERT: u32 = 0xff63;
    const XK_SPACE: u32 = 0x20;

    let raw = keysym.raw();
    match raw {
        XK_BACKSPACE => Some(egui::Key::Backspace),
        XK_TAB => Some(egui::Key::Tab),
        XK_RETURN => Some(egui::Key::Enter),
        XK_ESCAPE => Some(egui::Key::Escape),
        XK_HOME => Some(egui::Key::Home),
        XK_LEFT => Some(egui::Key::ArrowLeft),
        XK_UP => Some(egui::Key::ArrowUp),
        XK_RIGHT => Some(egui::Key::ArrowRight),
        XK_DOWN => Some(egui::Key::ArrowDown),
        XK_PAGE_UP => Some(egui::Key::PageUp),
        XK_PAGE_DOWN => Some(egui::Key::PageDown),
        XK_END => Some(egui::Key::End),
        XK_DELETE => Some(egui::Key::Delete),
        XK_INSERT => Some(egui::Key::Insert),
        XK_SPACE => Some(egui::Key::Space),
        _ => {
            if (0x20..=0x7e).contains(&raw) {
                egui_key_from_char(char::from(u8::try_from(raw).ok()?))
            } else {
                keysym.character().ok().and_then(egui_key_from_char)
            }
        }
    }
}

fn egui_key_from_char(character: char) -> Option<egui::Key> {
    match character.to_ascii_uppercase() {
        'A' => Some(egui::Key::A),
        'B' => Some(egui::Key::B),
        'C' => Some(egui::Key::C),
        'D' => Some(egui::Key::D),
        'E' => Some(egui::Key::E),
        'F' => Some(egui::Key::F),
        'G' => Some(egui::Key::G),
        'H' => Some(egui::Key::H),
        'I' => Some(egui::Key::I),
        'J' => Some(egui::Key::J),
        'K' => Some(egui::Key::K),
        'L' => Some(egui::Key::L),
        'M' => Some(egui::Key::M),
        'N' => Some(egui::Key::N),
        'O' => Some(egui::Key::O),
        'P' => Some(egui::Key::P),
        'Q' => Some(egui::Key::Q),
        'R' => Some(egui::Key::R),
        'S' => Some(egui::Key::S),
        'T' => Some(egui::Key::T),
        'U' => Some(egui::Key::U),
        'V' => Some(egui::Key::V),
        'W' => Some(egui::Key::W),
        'X' => Some(egui::Key::X),
        'Y' => Some(egui::Key::Y),
        'Z' => Some(egui::Key::Z),
        '0' => Some(egui::Key::Num0),
        '1' => Some(egui::Key::Num1),
        '2' => Some(egui::Key::Num2),
        '3' => Some(egui::Key::Num3),
        '4' => Some(egui::Key::Num4),
        '5' => Some(egui::Key::Num5),
        '6' => Some(egui::Key::Num6),
        '7' => Some(egui::Key::Num7),
        '8' => Some(egui::Key::Num8),
        '9' => Some(egui::Key::Num9),
        _ => None,
    }
}

fn is_printable_char(character: char) -> bool {
    !character.is_ascii_control()
}

fn is_cut_command(modifiers: egui::Modifiers, key: egui::Key) -> bool {
    key == egui::Key::Cut || (modifiers.command && key == egui::Key::X)
}

fn is_copy_command(modifiers: egui::Modifiers, key: egui::Key) -> bool {
    key == egui::Key::Copy || (modifiers.command && key == egui::Key::C)
}

fn is_paste_command(modifiers: egui::Modifiers, key: egui::Key) -> bool {
    key == egui::Key::Paste || (modifiers.command && key == egui::Key::V)
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use ctb_workspace_x11_client::{X11Capability, unsupported};

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
