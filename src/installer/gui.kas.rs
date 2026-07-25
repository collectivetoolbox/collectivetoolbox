#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

use kas::window::Window;
use kas_widgets::Button;

pub fn run_installer() -> anyhow::Result<()> {
    ctb_utilities::debug!("Creating window");
    let ui = kas_widgets::column![
        "Hello, world!",
        Button::label("&Close").with(|cx, _| cx.exit())
    ];
    let window = Window::new(ui, "Hello").escapable();

    ctb_utilities::debug!("Created window");
    let res = kas::runner::Runner::new(())?.with(window).run();
    ctb_utilities::debug_fmt!("Done running: {res:?}");
    Ok(())
}
