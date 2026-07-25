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
