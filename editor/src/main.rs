use editor::{RgEditor, generate_nodes};
use gpui::*;

fn main() {
    Application::new().run(|cx: &mut App| {
        let mut window_size = size(px(1600.0), px(1200.0));
        if let Some(display) = cx.primary_display() {
            let display_size = display.bounds().size;
            window_size.width = window_size.width.min(display_size.width * 0.85);
            window_size.height = window_size.height.min(display_size.height * 0.85);
        }

        let window_bounds = Bounds::centered(None, window_size, cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            window_min_size: Some(gpui::Size {
                width: px(200.),
                height: px(200.),
            }),
            kind: WindowKind::Normal,
            ..Default::default()
        };

        cx.open_window(options, |_, cx| {
            cx.new(|cx| {
                let node_count = 4;
                let nodes = generate_nodes(node_count);
                RgEditor::new(cx, nodes)
            })
        })
        .unwrap();
    });
}
