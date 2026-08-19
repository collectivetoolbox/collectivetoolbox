// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Native desktop webview embedding and window integration.

#[cfg(not(target_os = "linux"))]
#[derive(Default)]
struct App {
    url: String,
    window: Option<Window>,
    webview: Option<wry::WebView>,
}

#[cfg(not(target_os = "linux"))]
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let webview = WebViewBuilder::new()
            .with_url(&self.url)
            .build(&window)
            .unwrap();

        self.window = Some(window);
        self.webview = Some(webview);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
    }
}

pub fn start_webview(url: String) {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            // There was a crash here when it opened due to XKB_CONFIG_ROOT being set to a directory that did not have the necessary evdev etc. configuration built correctly. I tried temporarily unsetting XKB_CONFIG_ROOT around this call but it didn't seem to work. Fixing the built data should work though.
            if webbrowser::open(&url).is_err() {
                eprintln!("Failed to open web browser to URL: {url}");
            }
        } else {
            let event_loop = EventLoop::new().unwrap();
            let mut app = App::default();
            app.url = url;
            event_loop.run_app(&mut app).unwrap();
        }
    }
}
