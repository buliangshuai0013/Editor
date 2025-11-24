use gpui::*;

//************************************************************************** RgRect **********************************************************************************//
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    Body,
    None,
}

pub struct RgRect {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub drag_offset: Option<(f32, f32)>,
    pub resize_handle: ResizeHandle,
    pub zoom: f32,
    pub pan: (f32, f32),
    pub selected: bool,
    pub initial_drag_data: Option<(f32, f32, f32, f32)>,
    pub is_dragging: bool,
    pub is_resizing: bool,
    pub current_mouse_position: Option<Point<Pixels>>,
    pub show_handles: bool,
}

//************************************************************************** Trait **********************************************************************************//
impl RgRect {
    pub fn new(id: u64, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            drag_offset: None,
            resize_handle: ResizeHandle::None,
            zoom: 1.0,
            pan: (0.0, 0.0),
            selected: false,
            initial_drag_data: None,
            is_dragging: false,
            is_resizing: false,
            current_mouse_position: None,
            show_handles: true,
        }
    }

    pub fn screen_position(&self) -> (f32, f32) {
        (
            self.pan.0 + self.x * self.zoom,
            self.pan.1 + self.y * self.zoom,
        )
    }

    pub fn screen_size(&self) -> (f32, f32) {
        (self.width * self.zoom, self.height * self.zoom)
    }

    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        (
            self.pan.0 + world_x * self.zoom,
            self.pan.1 + world_y * self.zoom,
        )
    }

    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        (
            (screen_x - self.pan.0) / self.zoom,
            (screen_y - self.pan.1) / self.zoom,
        )
    }

    pub fn get_handle_position(&self, handle: ResizeHandle) -> (f32, f32) {
        let (screen_x, screen_y) = self.screen_position();
        let (screen_width, screen_height) = self.screen_size();

        match handle {
            ResizeHandle::TopLeft => (screen_x, screen_y),
            ResizeHandle::Top => (screen_x + screen_width / 2.0, screen_y),
            ResizeHandle::TopRight => (screen_x + screen_width, screen_y),
            ResizeHandle::Right => (screen_x + screen_width, screen_y + screen_height / 2.0),
            ResizeHandle::BottomRight => (screen_x + screen_width, screen_y + screen_height),
            ResizeHandle::Bottom => (screen_x + screen_width / 2.0, screen_y + screen_height),
            ResizeHandle::BottomLeft => (screen_x, screen_y + screen_height),
            ResizeHandle::Left => (screen_x, screen_y + screen_height / 2.0),
            _ => (0.0, 0.0),
        }
    }

    pub fn detect_handle_at(&self, position: Point<Pixels>) -> ResizeHandle {
        let screen_x = position.x.to_f64() as f32;
        let screen_y = position.y.to_f64() as f32;

        let (node_screen_x, node_screen_y) = self.screen_position();
        let (node_screen_width, node_screen_height) = self.screen_size();

        // 如果节点未选中，只检查是否在矩形内部
        if !self.selected {
            let is_inside = screen_x >= node_screen_x
                && screen_x <= node_screen_x + node_screen_width
                && screen_y >= node_screen_y
                && screen_y <= node_screen_y + node_screen_height;

            return if is_inside {
                ResizeHandle::Body
            } else {
                ResizeHandle::None
            };
        }

        // 多选时禁止调整大小，只允许移动
        // 这里需要从外部获取是否是多选状态，但我们可以通过检查 show_handles 来判断
        // 在 RgEditor 中，多选时会将 show_handles 设置为 false
        if !self.show_handles {
            let is_inside = screen_x >= node_screen_x
                && screen_x <= node_screen_x + node_screen_width
                && screen_y >= node_screen_y
                && screen_y <= node_screen_y + node_screen_height;

            return if is_inside {
                ResizeHandle::Body
            } else {
                ResizeHandle::None
            };
        }

        // 节点选中且允许显示手柄时，检查手柄
        let handle_size = 12.0 * self.zoom.max(1.0);
        let half_handle = handle_size / 2.0;

        let handles = [
            (ResizeHandle::TopLeft, (node_screen_x, node_screen_y)),
            (ResizeHandle::Top, (node_screen_x + node_screen_width / 2.0, node_screen_y)),
            (ResizeHandle::TopRight, (node_screen_x + node_screen_width, node_screen_y)),
            (ResizeHandle::Right, (node_screen_x + node_screen_width, node_screen_y + node_screen_height / 2.0)),
            (ResizeHandle::BottomRight, (node_screen_x + node_screen_width, node_screen_y + node_screen_height)),
            (ResizeHandle::Bottom, (node_screen_x + node_screen_width / 2.0, node_screen_y + node_screen_height)),
            (ResizeHandle::BottomLeft, (node_screen_x, node_screen_y + node_screen_height)),
            (ResizeHandle::Left, (node_screen_x, node_screen_y + node_screen_height / 2.0)),
        ];

        // 检查是否在任何一个手柄上
        for (handle, (hx, hy)) in handles {
            if screen_x >= hx - half_handle
                && screen_x <= hx + half_handle
                && screen_y >= hy - half_handle
                && screen_y <= hy + half_handle
            {
                return handle;
            }
        }

        // 检查是否在边缘区域（扩展检测范围）
        let edge_tolerance = 8.0 * self.zoom.max(1.0);

        // 检查左边缘（包括外部区域）
        if screen_x >= node_screen_x - edge_tolerance && screen_x <= node_screen_x + edge_tolerance
            && screen_y >= node_screen_y - edge_tolerance && screen_y <= node_screen_y + node_screen_height + edge_tolerance {
            return ResizeHandle::Left;
        }
        // 检查右边缘
        if screen_x >= node_screen_x + node_screen_width - edge_tolerance
            && screen_x <= node_screen_x + node_screen_width + edge_tolerance
            && screen_y >= node_screen_y - edge_tolerance && screen_y <= node_screen_y + node_screen_height + edge_tolerance {
            return ResizeHandle::Right;
        }
        // 检查上边缘
        if screen_y >= node_screen_y - edge_tolerance && screen_y <= node_screen_y + edge_tolerance
            && screen_x >= node_screen_x - edge_tolerance && screen_x <= node_screen_x + node_screen_width + edge_tolerance {
            return ResizeHandle::Top;
        }
        // 检查下边缘
        if screen_y >= node_screen_y + node_screen_height - edge_tolerance
            && screen_y <= node_screen_y + node_screen_height + edge_tolerance
            && screen_x >= node_screen_x - edge_tolerance && screen_x <= node_screen_x + node_screen_width + edge_tolerance {
            return ResizeHandle::Bottom;
        }

        // 最后检查是否在矩形内部（用于移动）
        let is_inside = screen_x >= node_screen_x
            && screen_x <= node_screen_x + node_screen_width
            && screen_y >= node_screen_y
            && screen_y <= node_screen_y + node_screen_height;

        if is_inside {
            ResizeHandle::Body
        } else {
            ResizeHandle::None
        }
    }
    pub fn get_cursor_for_handle(handle: ResizeHandle) -> CursorStyle {
        match handle {
            ResizeHandle::TopLeft | ResizeHandle::BottomRight => CursorStyle::ResizeUpLeftDownRight,
            ResizeHandle::TopRight | ResizeHandle::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
            ResizeHandle::Top | ResizeHandle::Bottom => CursorStyle::ResizeUpDown,
            ResizeHandle::Left | ResizeHandle::Right => CursorStyle::ResizeLeftRight,
            ResizeHandle::Body => CursorStyle::PointingHand,
            ResizeHandle::None => CursorStyle::Arrow,
        }
    }

    // 处理拖拽开始
    pub fn start_drag(&mut self, position: Point<Pixels>, handle: ResizeHandle) {
        let screen_x = position.x.to_f64() as f32;
        let screen_y = position.y.to_f64() as f32;
        let (world_x, world_y) = self.screen_to_world(screen_x, screen_y);

        match handle {
            ResizeHandle::Body => {
                self.is_dragging = true;
                self.drag_offset = Some((world_x - self.x, world_y - self.y));
            }

            handle if handle != ResizeHandle::None => {
                self.is_resizing = true;
                self.resize_handle = handle;
                self.initial_drag_data = Some((self.x, self.y, self.width, self.height));
            }
            _ => {}
        }
    }

    // 处理拖拽更新
    pub fn update_drag(&mut self, position: Point<Pixels>) {
        let screen_x = position.x.to_f64() as f32;
        let screen_y = position.y.to_f64() as f32;
        let (world_x, world_y) = self.screen_to_world(screen_x, screen_y);

        if self.is_dragging {
            if let Some((offset_x, offset_y)) = self.drag_offset {
                self.x = world_x - offset_x;
                self.y = world_y - offset_y;
            }
        } else if self.is_resizing {
            if let Some((initial_x, initial_y, initial_width, initial_height)) = self.initial_drag_data {
                match self.resize_handle {
                    ResizeHandle::TopLeft => {
                        let new_width = initial_width + (initial_x - world_x);
                        let new_height = initial_height + (initial_y - world_y);
                        if new_width > 10.0 && new_height > 10.0 {
                            self.x = world_x;
                            self.y = world_y;
                            self.width = new_width;
                            self.height = new_height;
                        }
                    }

                    ResizeHandle::Top => {
                        let new_height = initial_height + (initial_y - world_y);
                        if new_height > 10.0 {
                            self.y = world_y;
                            self.height = new_height;
                        }
                    }

                    ResizeHandle::TopRight => {
                        let new_width = world_x - initial_x;
                        let new_height = initial_height + (initial_y - world_y);
                        if new_width > 10.0 && new_height > 10.0 {
                            self.y = world_y;
                            self.width = new_width;
                            self.height = new_height;
                        }
                    }

                    ResizeHandle::Right => {
                        let new_width = world_x - initial_x;
                        if new_width > 10.0 {
                            self.width = new_width;
                        }
                    }

                    ResizeHandle::BottomRight => {
                        let new_width = world_x - initial_x;
                        let new_height = world_y - initial_y;
                        if new_width > 10.0 && new_height > 10.0 {
                            self.width = new_width;
                            self.height = new_height;
                        }
                    }

                    ResizeHandle::Bottom => {
                        let new_height = world_y - initial_y;
                        if new_height > 10.0 {
                            self.height = new_height;
                        }
                    }

                    ResizeHandle::BottomLeft => {
                        let new_width = initial_width + (initial_x - world_x);
                        let new_height = world_y - initial_y;
                        if new_width > 10.0 && new_height > 10.0 {
                            self.x = world_x;
                            self.width = new_width;
                            self.height = new_height;
                        }
                    }

                    ResizeHandle::Left => {
                        let new_width = initial_width + (initial_x - world_x);
                        if new_width > 10.0 {
                            self.x = world_x;
                            self.width = new_width;
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.is_resizing = false;
        self.drag_offset = None;
        self.initial_drag_data = None;
        self.resize_handle = ResizeHandle::None;
    }

    pub fn update_mouse_position(&mut self, position: Point<Pixels>) -> ResizeHandle {
        self.current_mouse_position = Some(position);
        self.detect_handle_at(position)
    }

    pub fn set_show_handles(&mut self, show: bool) {
        self.show_handles = show;
    }
}

//************************************************************************** Render **********************************************************************************//
impl Render for RgRect {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (screen_x, screen_y) = self.screen_position();
        let (screen_width, screen_height) = self.screen_size();

        let border_color = if self.selected {
            rgb(0x00ff00)
        } else {
            rgb(0x505050)
        };

        let handle_size = 12.0 * self.zoom.max(1.0);
        let half_handle = handle_size / 2.0;

        let base_font_size = 14.0;
        let scaled_font_size = base_font_size * self.zoom;

        let base_rect = div()
            .absolute()
            .left(px(screen_x))
            .top(px(screen_y))
            .w(px(screen_width))
            .h(px(screen_height))
            .bg(rgba(0xf0f0f0aa))
            .border_1()
            .border_color(border_color)
            .text_color(black())
            .text_size(px(scaled_font_size))
            .line_height(px(scaled_font_size * 1.2))
            .flex()
            .items_center()
            .justify_center()
            .child(format!("Rect {}", self.id));

        if !self.selected {
            return base_rect;
        }

        let mut container = div().absolute().size_full();

        container = container.child(base_rect);

        let edge_highlight_color = rgba(0x008aff88);
        let edge_size = 2.0 * self.zoom.max(0.5);

        container = container.child(
            div()
                .absolute()
                .left(px(screen_x))
                .top(px(screen_y))
                .w(px(screen_width))
                .h(px(edge_size))
                .bg(edge_highlight_color),
        );

        container = container.child(
            div()
                .absolute()
                .left(px(screen_x))
                .top(px(screen_y + screen_height - edge_size))
                .w(px(screen_width))
                .h(px(edge_size))
                .bg(edge_highlight_color),
        );

        container = container.child(
            div()
                .absolute()
                .left(px(screen_x))
                .top(px(screen_y))
                .w(px(edge_size))
                .h(px(screen_height))
                .bg(edge_highlight_color),
        );

        container = container.child(
            div()
                .absolute()
                .left(px(screen_x + screen_width - edge_size))
                .top(px(screen_y))
                .w(px(edge_size))
                .h(px(screen_height))
                .bg(edge_highlight_color),
        );

        if self.show_handles {
            let handles = [
                (ResizeHandle::TopLeft, (screen_x, screen_y)),
                (ResizeHandle::Top, (screen_x + screen_width / 2.0, screen_y)),
                (ResizeHandle::TopRight, (screen_x + screen_width, screen_y)),
                (ResizeHandle::Right, (screen_x + screen_width, screen_y + screen_height / 2.0)),
                (ResizeHandle::BottomRight, (screen_x + screen_width, screen_y + screen_height)),
                (ResizeHandle::Bottom, (screen_x + screen_width / 2.0, screen_y + screen_height)),
                (ResizeHandle::BottomLeft, (screen_x, screen_y + screen_height)),
                (ResizeHandle::Left, (screen_x, screen_y + screen_height / 2.0)),
            ];

            for (handle, (hx, hy)) in handles {
                let handle_color = match handle {
                    ResizeHandle::TopLeft |
                    ResizeHandle::TopRight |
                    ResizeHandle::BottomLeft |
                    ResizeHandle::BottomRight => rgb(0xff0000),

                    ResizeHandle::Top |
                    ResizeHandle::Bottom |
                    ResizeHandle::Left |
                    ResizeHandle::Right => rgb(0x0000ff),

                    _ => rgb(0xffffff),
                };

                container = container.child(
                    div()
                        .absolute()
                        .left(px(hx - half_handle))
                        .top(px(hy - half_handle))
                        .w(px(handle_size))
                        .h(px(handle_size))
                        .bg(handle_color)
                        .border_1()
                        .border_color(rgb(0x000000))
                        .rounded(px(handle_size / 4.0))
                );
            }
        }

        container
    }
}
