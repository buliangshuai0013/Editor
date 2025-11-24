use gpui::*;
use crate::*;

//************************************************************************** RgEditor **********************************************************************************//
pub struct RgEditor {
    bounds: Bounds<Pixels>,
    nodes: Vec<Entity<RgRect>>,
    zoom: f32,
    base_size: Size<Pixels>,
    pan: Point<Pixels>,
    is_selecting: bool,
    drag_state: Option<(Vec<Entity<RgRect>>, ResizeHandle)>,
    selection_rect: Option<(Point<Pixels>, Point<Pixels>)>,
    click_start_position: Option<Point<Pixels>>,
    content_bounds: (f32, f32, f32, f32),
    view_initialized: bool,
    is_updating_bounds: bool,
    user_zoomed: bool,
}

//************************************************************************** Trait **********************************************************************************//
impl RgEditor {
    pub fn new(cx: &mut App, nodes: Vec<RgRect>) -> Self {
        let zoom = 1.0;
        let pan = point(px(0.0), px(0.0));
        let mut node_entities: Vec<Entity<RgRect>> = Vec::with_capacity(nodes.len());

        for mut node in nodes {
            node.zoom = zoom;
            node.pan = (pan.x.to_f64() as f32, pan.y.to_f64() as f32);
            node_entities.push(cx.new(|_| node));
        }

        Self {
            bounds: Bounds::default(),
            nodes: node_entities,
            zoom,
            base_size: size(px(0.0), px(0.0)),
            pan,

            is_selecting: false,
            drag_state: None,
            selection_rect: None,
            click_start_position: None,
            content_bounds: (0.0, 0.0, 800.0, 600.0),
            view_initialized: false,
            is_updating_bounds: false,
            user_zoomed: false,
        }
    }

    pub fn set_content_bounds(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.content_bounds = (min_x, min_y, max_x, max_y);
    }

    fn update_base_size(&mut self, new_bounds: Bounds<Pixels>) {
        if self.is_updating_bounds {
            return;
        }

        self.is_updating_bounds = true;
        self.base_size = new_bounds.size;

        if !self.view_initialized || !self.user_zoomed {
            self.adjust_view_to_fit_content_bounds();
            self.view_initialized = true;
        }

        self.is_updating_bounds = false;
    }

    fn adjust_view_to_fit_content_bounds(&mut self) {
        if self.base_size.width > px(0.0) && self.base_size.height > px(0.0) {
            let (min_x, min_y, max_x, max_y) = self.content_bounds;
            let content_width = max_x - min_x;
            let content_height = max_y - min_y;
            let scale_x = self.base_size.width.to_f64() as f32 / content_width;
            let scale_y = self.base_size.height.to_f64() as f32 / content_height;

            let new_zoom = if scale_x < 1.0 || scale_y < 1.0 {
                scale_x.min(scale_y)
            } else {
                1.0
            }.clamp(0.1, 4.0);

            self.zoom = new_zoom;

            let content_center_x = (min_x + max_x) / 2.0;
            let content_center_y = (min_y + max_y) / 2.0;
            let view_center_x = self.base_size.width.to_f64() as f32 / 2.0;
            let view_center_y = self.base_size.height.to_f64() as f32 / 2.0;

            self.pan = point(
                px(view_center_x - content_center_x * new_zoom),
                px(view_center_y - content_center_y * new_zoom),
            );
        }
    }

    fn update_nodes_pan_zoom(&mut self, cx: &mut Context<Self>) {
        for n in &self.nodes {
            let pan = self.pan;
            let zoom = self.zoom;
            cx.update_entity(n, move |node, _| {
                node.zoom = zoom;
                node.pan = (pan.x.to_f64() as f32, pan.y.to_f64() as f32);
            });
        }
    }

    fn get_selected_nodes(&self, cx: &mut Context<Self>) -> Vec<Entity<RgRect>> {
        self.nodes.iter()
            .filter(|node_entity| {
                cx.read_entity(node_entity, |node, _| node.selected)
            })
            .cloned()
            .collect()
    }

    fn is_node_in_selection_rect(&self, node_entity: &Entity<RgRect>, selection_rect: &(Point<Pixels>, Point<Pixels>), cx: &mut Context<Self>) -> bool {
        let (start, end) = selection_rect;

        cx.read_entity(node_entity, |node, _| {
            let (node_screen_x, node_screen_y) = node.screen_position();
            let (node_screen_width, node_screen_height) = node.screen_size();

            let rect_left = start.x.min(end.x);
            let rect_right = start.x.max(end.x);
            let rect_top = start.y.min(end.y);
            let rect_bottom = start.y.max(end.y);

            node_screen_x < rect_right.to_f64() as f32 &&
            node_screen_x + node_screen_width > rect_left.to_f64() as f32 &&
            node_screen_y < rect_bottom.to_f64() as f32 &&
            node_screen_y + node_screen_height > rect_top.to_f64() as f32
        })
    }

    fn get_nodes_at_position_with_edges(&self, position: Point<Pixels>, cx: &mut Context<Self>) -> Vec<Entity<RgRect>> {
        let mut nodes_at_position = Vec::new();

        for node_entity in self.nodes.iter().rev() {
            let is_hit = cx.read_entity(node_entity, |node, _| {
                let handle = node.detect_handle_at(position);
                handle != ResizeHandle::None
            });

            if is_hit {
                nodes_at_position.push(node_entity.clone());
            }
        }

        nodes_at_position
    }

    fn smart_select_nodes(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) -> Option<Entity<RgRect>> {
        let nodes_at_position = self.get_nodes_at_position_with_edges(position, cx);

        if nodes_at_position.is_empty() {
            return None;
        }

        let currently_selected = self.get_selected_nodes(cx);
        let currently_selected_at_position: Vec<Entity<RgRect>> = currently_selected
            .into_iter()
            .filter(|node| nodes_at_position.contains(node))
            .collect();

        if !currently_selected_at_position.is_empty() {
            let current_selected = &currently_selected_at_position[0];
            let current_index = nodes_at_position.iter().position(|n| n == current_selected).unwrap();
            let next_index = (current_index + 1) % nodes_at_position.len();
            return Some(nodes_at_position[next_index].clone());
        }

        Some(nodes_at_position[0].clone())
    }

    fn bring_selected_nodes_to_front(&mut self, cx: &mut Context<Self>) {
        let selected_nodes = self.get_selected_nodes(cx);

        if selected_nodes.is_empty() {
            return;
        }

        let mut new_order: Vec<Entity<RgRect>> = Vec::with_capacity(self.nodes.len());

        for node in &self.nodes {
            if !selected_nodes.contains(node) {
                new_order.push(node.clone());
            }
        }

        for node in selected_nodes {
            new_order.push(node);
        }

        self.nodes = new_order;
    }

    fn show_select_handles(&mut self, cx: &mut Context<Self>) {
        let selected_nodes = self.get_selected_nodes(cx);
        for node_entity in &selected_nodes {
            cx.update_entity(node_entity, |node, _| {
                node.set_show_handles(selected_nodes.len() <= 1);
            });
        }
    }

    fn clear_all_select(&mut self, cx: &mut Context<Self>) {
        for node_entity in &self.nodes {
            cx.update_entity(node_entity, |node, _| {
                node.set_show_handles(true);
            });
        }

        for node_entity in &self.nodes {
            cx.update_entity(node_entity, |node, _| {
                node.selected = false;
            });
        }
    }

    fn toggle_node_selection(&mut self, node_entity: &Entity<RgRect>, cx: &mut Context<Self>) {
        cx.update_entity(node_entity, |node, _| {
            node.selected = !node.selected;
        });
    }

    fn on_mouse_left_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let position = event.position - self.bounds.origin;

        self.click_start_position = Some(position);

        //***************1.如果按shift*********************//
        if event.modifiers.shift {
            if let Some(node_entity) = self.smart_select_nodes(position, cx) {
                self.toggle_node_selection(&node_entity, cx);
                self.show_select_handles(cx);
                self.bring_selected_nodes_to_front(cx);
            }

            cx.refresh_windows();
            return;
        }

        //***************2.如果点击手柄*********************//
        let mut hit_handle_node = None;
        let mut hit_handle_type = ResizeHandle::None;

        for node_entity in &self.nodes {
            let handle = cx.read_entity(node_entity, |node, _| {
                node.detect_handle_at(position)
            });

            if handle != ResizeHandle::None && handle != ResizeHandle::Body {
                hit_handle_node = Some(node_entity.clone());
                hit_handle_type = handle;
                break;
            }
        }

        if let Some(node_entity) = hit_handle_node {
            cx.update_entity(&node_entity, |node, _| {
                node.start_drag(position, hit_handle_type);
            });
            self.drag_state = Some((vec![node_entity], hit_handle_type));
            cx.refresh_windows();
            return;
        }

        //*************** 3.点击物体 *********************//
        if let Some(selected_node) = self.smart_select_nodes(position, cx) {
            let is_already_selected = cx.read_entity(&selected_node, |node, _| node.selected);
            if !is_already_selected {
                for node in &self.nodes {
                    let should_select = node == &selected_node;
                    cx.update_entity(node, |node, _| {
                        node.selected = should_select;
                    });
                }
            }

            self.bring_selected_nodes_to_front(cx);
            self.show_select_handles(cx);
            cx.refresh_windows();
            return;
        }

        //*************** 4.点击空白处开始框选 *********************//
        self.is_selecting = true;
        self.selection_rect = Some((position, position));
        self.clear_all_select(cx);
        cx.refresh_windows();
    }

    fn on_mouse_left_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.is_selecting = false;
            self.selection_rect = None;
            self.show_select_handles(cx);
        }

        if let Some((selected_nodes, _)) = &self.drag_state {
            for node_entity in selected_nodes {
                cx.update_entity(node_entity, |node, _| {
                    node.end_drag();
                });
            }
            self.drag_state = None;
        }

        self.click_start_position = None;
        cx.refresh_windows();
    }

    fn on_mouse_right_down(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_all_select(cx);
        cx.refresh_windows();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let position = event.position - self.bounds.origin;

        //****************************** 1.框选 ****************************//
        if self.is_selecting {
            if let Some((start, _)) = &mut self.selection_rect {
                self.selection_rect = Some((*start, position));

                let selection_rect = self.selection_rect.unwrap();

                for node_entity in &self.nodes {
                    let is_selected = self.is_node_in_selection_rect(node_entity, &selection_rect, cx);
                    cx.update_entity(node_entity, |node, _| {
                        node.selected = is_selected;
                    });
                }
            }
            cx.refresh_windows();
            return;
        }

        //****************************** 2.开始拖动 ****************************//
        if let Some(_click_start) = self.click_start_position {
            if self.drag_state.is_none() {
                let selected_nodes = self.get_selected_nodes(cx);
                if !selected_nodes.is_empty() {
                    let nodes_to_drag = selected_nodes.clone();
                    for node_entity in &nodes_to_drag {
                        cx.update_entity(node_entity, |node, _| {
                            node.start_drag(position, ResizeHandle::Body);
                        });
                    }
                    self.drag_state = Some((nodes_to_drag, ResizeHandle::Body));
                }

                cx.refresh_windows();
                return;
            }
        }

        //****************************** 3.进行拖动 ****************************//
        if let Some((selected_nodes, _handle)) = &self.drag_state {
            for node_entity in selected_nodes {
                cx.update_entity(node_entity, |node, _| {
                    node.update_drag(position);
                });
            }
        }

        cx.refresh_windows();
    }

    fn on_bounds_changed(&mut self, new_bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        let old_bounds = self.bounds;
        self.bounds = new_bounds;

        if old_bounds.size != new_bounds.size {
            self.update_base_size(new_bounds);
            self.update_nodes_pan_zoom(cx);
            cx.refresh_windows();
        }
    }
}

//************************************************************************** Render **********************************************************************************//
impl Render for RgEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().clone();
        let selection_rect = self.selection_rect.clone();
        let content_bounds = self.content_bounds;
        let zoom = self.zoom;
        let pan = self.pan;

        let mut element =
            div()
            .size_full()
            .relative()
            .bg(rgb(0xffffff))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_left_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_left_up))
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_mouse_right_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .children(self.nodes.iter().cloned())
            .child({
                canvas(
                    move |bounds, _, cx| {
                        view.update(cx, |r, cx| {
                            if r.bounds != bounds {
                                r.on_bounds_changed(bounds, cx);
                            } else {
                                r.bounds = bounds;
                            }
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            });

        let (min_x, min_y, max_x, max_y) = content_bounds;
        let screen_min_x = pan.x + px(min_x * zoom);
        let screen_min_y = pan.y + px(min_y * zoom);
        let screen_max_x = pan.x + px(max_x * zoom);
        let screen_max_y = pan.y + px(max_y * zoom);
        let screen_width = screen_max_x - screen_min_x;
        let screen_height = screen_max_y - screen_min_y;

        element = element.child(
            div()
                .absolute()
                .left(screen_min_x)
                .top(screen_min_y)
                .w(screen_width)
                .h(screen_height)
                .border_2()
                .border_color(rgba(0x0000ff88))
                .border_dashed()
                .bg(rgba(0xff000010))
        );

        if let Some((start, end)) = selection_rect {
            let rect_left = start.x.min(end.x);
            let rect_right = start.x.max(end.x);
            let rect_top = start.y.min(end.y);
            let rect_bottom = start.y.max(end.y);
            let rect_width = rect_right - rect_left;
            let rect_height = rect_bottom - rect_top;

            element = element.child(
                div()
                    .relative()
                    .left(px(rect_left.to_f64() as f32))
                    .top(px(rect_top.to_f64() as f32))
                    .w(px(rect_width.to_f64() as f32))
                    .h(px(rect_height.to_f64() as f32))
                    .bg(rgba(0x0000ff55))
                    .border_1()
                    .border_color(rgb(0x0000ff))
            );
        }

        element
    }
}
