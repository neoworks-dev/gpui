// Validation harness for the shell branch: opens one layer surface on the only display of
// logical size LAYER_CHECK_SIZE (`960x540`), resizes it, toggles keyboard interactivity, and prints what the
// window sees. LAYER_CHECK_SCENARIO picks the request: `bar`, `full` or `sized`.

fn main() {
    #[cfg(all(target_os = "linux", feature = "wayland"))]
    check::main();
}

#[cfg(all(target_os = "linux", feature = "wayland"))]
mod check {
    use std::time::Duration;

    use gpui::{
        App, Bounds, Context, Size, Window, WindowBackgroundAppearance, WindowBounds, WindowKind,
        WindowOptions, div, layer_shell::*, point, prelude::*, px, rgba,
    };
    use gpui_platform::application;

    struct Check {
        step: usize,
    }

    impl Render for Check {
        fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let size = window.viewport_size();
            println!("step {} viewport {}x{}", self.step, size.width, size.height);
            div().size_full().bg(rgba(0x3366ffcc))
        }
    }

    fn scenario() -> (Anchor, Size<gpui::Pixels>, Size<gpui::Pixels>) {
        match std::env::var("LAYER_CHECK_SCENARIO").as_deref() {
            Ok("full") => (
                Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                Size::new(px(0.), px(0.)),
                Size::new(px(0.), px(0.)),
            ),
            Ok("sized") => (Anchor::TOP, Size::new(px(300.), px(50.)), Size::new(px(400.), px(60.))),
            _ => (
                Anchor::LEFT | Anchor::RIGHT | Anchor::TOP,
                Size::new(px(0.), px(38.)),
                Size::new(px(0.), px(80.)),
            ),
        }
    }

    pub fn main() {
        application().run(|cx: &mut App| {
            let target_size = std::env::var("LAYER_CHECK_SIZE").unwrap_or_default();
            let matching: Vec<_> = cx
                .displays()
                .into_iter()
                .filter(|display| {
                    let size = display.bounds().size;
                    format!("{}x{}", f32::from(size.width), f32::from(size.height)) == target_size
                })
                .collect();
            let [display] = matching.as_slice() else {
                println!("{} displays of size {target_size:?}; not opening anything", matching.len());
                std::process::exit(1);
            };
            let display_id = display.id();
            let display_id = Some(display_id);

            let (anchor, initial, resized) = scenario();
            let window = cx
                .open_window(
                    WindowOptions {
                        titlebar: None,
                        display_id,
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: point(px(0.), px(0.)),
                            size: initial,
                        })),
                        app_id: Some("gpui-layer-check".to_string()),
                        window_background: WindowBackgroundAppearance::Transparent,
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "gpui-layer-check".to_string(),
                            anchor,
                            keyboard_interactivity: KeyboardInteractivity::None,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    |_, cx| cx.new(|_| Check { step: 0 }),
                )
                .unwrap();

            cx.spawn(async move |cx| {
                cx.background_executor().timer(Duration::from_millis(1500)).await;
                let _ = window.update(cx, |check, window, cx| {
                    check.step = 1;
                    window.resize(resized);
                    cx.notify();
                });
                cx.background_executor().timer(Duration::from_millis(1500)).await;
                let _ = window.update(cx, |check, window, cx| {
                    check.step = 2;
                    window.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
                    cx.notify();
                });
            })
            .detach();
        });
    }
}
