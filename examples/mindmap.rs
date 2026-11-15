use std::sync::atomic::{AtomicU32, Ordering};

use gpui::prelude::FluentBuilder;
use gpui_flow::*;
use gpui_kit::gpui::*;

static NEXT_ID: AtomicU32 = AtomicU32::new(100);

fn next_id() -> String {
    NEXT_ID.fetch_add(1, Ordering::Relaxed).to_string()
}

type NodeDatum = ();

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub bg: gpui_kit::Hsla,
    pub grid: gpui_kit::Hsla,
    pub card: gpui_kit::Hsla,
    pub card_border: gpui_kit::Hsla,
    pub card_hover: gpui_kit::Hsla,
    pub text: gpui_kit::Hsla,
    pub text_dim: gpui_kit::Hsla,

    pub orange: gpui_kit::Hsla,
    pub blue: gpui_kit::Hsla,
    pub green: gpui_kit::Hsla,
}
const _: () = {
    impl Default for ColorScheme {
        fn default() -> Self {
            // Dark theme colors
            Self {
                bg: gpui_kit::rgb(0x1a1a2e).into(),
                grid: gpui_kit::rgb(0x2a2a3e).into(),
                card: gpui_kit::rgb(0x25253a).into(),
                card_border: gpui_kit::rgb(0x3a3a50).into(),
                card_hover: gpui_kit::rgb(0x30304a).into(),
                orange: gpui_kit::rgb(0xe8a87c).into(),
                blue: gpui_kit::rgb(0x7eb8da).into(),
                green: gpui_kit::rgb(0x82c991).into(),
                text: gpui_kit::rgb(0xf0f0f0).into(),
                text_dim: gpui_kit::rgb(0xb0b0b0).into(),
            }
        }
    }
};

struct MindMapApp {
    flow: Entity<FlowGraph<NodeDatum>>,
    state: Entity<FlowState<NodeDatum>>,
    minimap: Entity<Minimap<NodeDatum>>,
    focus_handle: FocusHandle,
    /// Which node is being edited.
    editing: Option<NodeId>,
    /// Text buffer during editing.
    edit_buffer: String,
}

impl MindMapApp {
    fn start_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.state.read(cx);
        if let Some(node) = state.nodes.iter().find(|n| n.selected) {
            self.editing = Some(node.id.clone());
            self.edit_buffer = node.label.to_string();
            // Take focus so FlowGraph stops handling delete keys
            self.focus_handle.focus(window, cx);
            cx.notify();
        }
    }

    fn finish_editing(&mut self, save: bool, cx: &mut Context<Self>) {
        if let Some(ref node_id) = self.editing {
            if save {
                let label: SharedString = self.edit_buffer.clone().into();
                let nid = node_id.clone();
                self.state.update(cx, |state, _| {
                    state.push_undo();
                    if let Some(node) = state.get_node_mut(&nid) {
                        node.label = label;
                    }
                });
            }
        }
        self.editing = None;
        self.edit_buffer.clear();
        cx.notify();
    }
}

impl Render for MindMapApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = ColorScheme::default();

        let editing = self.editing.clone();
        let edit_buffer = self.edit_buffer.clone();
        let _state_for_add = self.state.clone();

        div()
            .id("mindmap-root")
            .track_focus(&self.focus_handle)
            .size_full()
            .relative()
            .bg(colors.bg)
            .child(self.flow.clone())
            // Toolbar
            .child(
                div()
                    .absolute()
                    .top(px(12.0))
                    .left(px(12.0))
                    .flex()
                    .gap_2()
                    .child(toolbar_button("+ Branch", {
                        let s = self.state.clone();
                        move |_, _, cx| add_child_to_selected(&s, colors.orange, cx)
                    }))
                    .child(toolbar_button("+ Blue", {
                        let s = self.state.clone();
                        move |_, _, cx| add_child_to_selected(&s, colors.blue, cx)
                    }))
                    .child(toolbar_button("+ Green", {
                        let s = self.state.clone();
                        move |_, _, cx| add_child_to_selected(&s, colors.green, cx)
                    }))
                    .child(
                        div()
                            .ml_4()
                            .text_xs()
                            .text_color(colors.text_dim)
                            .child("Enter: edit | Del: delete | Cmd+Z: undo"),
                    ),
            )
            // Edit indicator
            .when(editing.is_some(), |el: Stateful<Div>| {
                el.child(
                    div()
                        .absolute()
                        .top(px(12.0))
                        .right(px(12.0))
                        .px_3()
                        .py_2()
                        .bg(colors.card)
                        .rounded_md()
                        .border_1()
                        .border_color(colors.orange)
                        .flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_dim)
                                .child("Editing:"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text)
                                .min_w(px(100.0))
                                .child(format!("{}|", edit_buffer)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_dim)
                                .child("Enter=save  Esc=cancel"),
                        ),
                )
            })
            // Minimap
            .child(
                div()
                    .absolute()
                    .bottom(px(12.0))
                    .right(px(12.0))
                    .child(self.minimap.clone()),
            )
            // Keyboard handler for editing
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let colors = ColorScheme::default();

                let key: &str = event.keystroke.key.as_ref();

                // If currently editing, handle text input
                if this.editing.is_some() {
                    if key == "enter" {
                        this.finish_editing(true, cx);
                        return;
                    }
                    if key == "escape" {
                        this.finish_editing(false, cx);
                        return;
                    }
                    if key == "backspace" {
                        this.edit_buffer.pop();
                        cx.notify();
                        return;
                    }
                    if key == "space" {
                        this.edit_buffer.push(' ');
                        cx.notify();
                        return;
                    }
                    // Type single characters
                    if key.len() == 1 {
                        let ch = key.chars().next().unwrap();
                        if event.keystroke.modifiers.shift {
                            this.edit_buffer.push(ch.to_ascii_uppercase());
                        } else {
                            this.edit_buffer.push(ch);
                        }
                        cx.notify();
                        return;
                    }
                    return;
                }

                // Not editing — handle shortcuts
                // Enter → start editing selected node
                if key == "enter" {
                    this.start_editing(window, cx);
                    return;
                }

                // Tab → add child to selected
                if key == "tab" {
                    add_child_to_selected(&this.state, colors.orange, cx);
                    cx.notify();
                }
            }))
    }
}

fn add_child_to_selected(
    state: &Entity<FlowState<NodeDatum>>,
    color: gpui_kit::Hsla,
    cx: &mut App,
) {
    state.update(cx, |state, _| {
        let parent = state.nodes.iter().find(|n| n.selected).cloned();
        let parent = match parent {
            Some(p) => p,
            None => return,
        };
        state.push_undo();
        let child_id: SharedString = next_id().into();
        let child_count = state.edges.iter().filter(|e| e.source == parent.id).count() as f32;
        let x = parent.position.x + 250.0;
        let y = parent.position.y + child_count * 60.0 - 20.0;
        let child = FlowNode::new(child_id.clone(), x, y)
            .label("New item")
            .node_type("branch")
            .handles(vec![
                HandleDef::target(HandlePosition::Left),
                HandleDef::source(HandlePosition::Right),
            ]);
        let edge_id: SharedString = format!("e{}-{}", parent.id, child_id).into();
        let edge = FlowEdge::new(edge_id, parent.id.clone(), child_id)
            .color(color)
            .stroke_width(3.0);
        state.nodes.push(child);
        state.edges.push(edge);
        state.rebuild_lookup();
    });
}

fn toolbar_button(
    label: &'static str,
    on_click: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let colors = ColorScheme::default();

    div()
        .id(SharedString::from(label))
        .px_3()
        .py_1()
        .bg(colors.card)
        .text_xs()
        .text_color(colors.text)
        .rounded_md()
        .border_1()
        .border_color(colors.card_border)
        .cursor(CursorStyle::PointingHand)
        .on_mouse_down(MouseButton::Left, on_click)
        .child(label)
}

/// Root node — large card with bold text.
fn render_root(node: &FlowNode<NodeDatum>, _window: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();

    div()
        .px_5()
        .py_3()
        .bg(colors.card)
        .rounded_xl()
        .border_1()
        .border_color(colors.card_border)
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::BOLD)
                .text_color(colors.text)
                .child(node_label(node)),
        )
        .into_any_element()
}

/// Branch node — medium card.
fn render_branch(node: &FlowNode<NodeDatum>, _window: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();

    div()
        .px_3()
        .py_2()
        .bg(colors.card)
        .rounded_lg()
        .border_1()
        .border_color(colors.card_border)
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.text)
                .child(node_label(node)),
        )
        .into_any_element()
}

/// Leaf node — small subtle card.
fn render_leaf(node: &FlowNode<NodeDatum>, _window: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();
    div()
        .px_3()
        .py_1()
        .bg(colors.card_hover)
        .rounded_md()
        .border_1()
        .border_color(colors.card_border)
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dim)
                .child(node_label(node)),
        )
        .into_any_element()
}

fn node_label(node: &FlowNode<NodeDatum>) -> String {
    if node.label.is_empty() {
        node.id.to_string()
    } else {
        node.label.to_string()
    }
}

fn main() {
    gpui_kit::application()
        // Application::with_platform(Rc::new(gpui_macos::MacPlatform::new(false)))
        .run(move |cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window, cx| {
                    let nodes = vec![
                        FlowNode::new("root", 80.0, 220.0)
                            .label("Weekend Plan")
                            .node_type("root")
                            .handles(vec![HandleDef::source(HandlePosition::Right)]),
                        // Categories
                        FlowNode::new("groceries", 350.0, 60.0)
                            .label("Groceries")
                            .node_type("branch")
                            .handles(vec![
                                HandleDef::target(HandlePosition::Left),
                                HandleDef::source(HandlePosition::Right),
                            ]),
                        FlowNode::new("errands", 350.0, 260.0)
                            .label("Errands")
                            .node_type("branch")
                            .handles(vec![
                                HandleDef::target(HandlePosition::Left),
                                HandleDef::source(HandlePosition::Right),
                            ]),
                        FlowNode::new("projects", 350.0, 420.0)
                            .label("Projects")
                            .node_type("branch")
                            .handles(vec![
                                HandleDef::target(HandlePosition::Left),
                                HandleDef::source(HandlePosition::Right),
                            ]),
                        // Groceries items
                        FlowNode::new("g1", 620.0, 0.0)
                            .label("Avocados")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("g2", 620.0, 50.0)
                            .label("Sourdough bread")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("g3", 620.0, 100.0)
                            .label("Oat milk")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("g4", 620.0, 150.0)
                            .label("Fresh basil")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        // Errands
                        FlowNode::new("e1", 620.0, 230.0)
                            .label("Return library books")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("e2", 620.0, 280.0)
                            .label("Pick up dry cleaning")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("e3", 620.0, 330.0)
                            .label("Post office")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        // Projects
                        FlowNode::new("p1", 620.0, 400.0)
                            .label("Fix kitchen shelf")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("p2", 620.0, 450.0)
                            .label("Repot succulents")
                            .node_type("leaf")
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                    ];

                    let colors = ColorScheme::default();
                    let edges = vec![
                        FlowEdge::new("e-r-g", "root", "groceries")
                            .color(colors.orange)
                            .stroke_width(3.0),
                        FlowEdge::new("e-r-e", "root", "errands")
                            .color(colors.blue)
                            .stroke_width(3.0),
                        FlowEdge::new("e-r-p", "root", "projects")
                            .color(colors.green)
                            .stroke_width(3.0),
                        FlowEdge::new("e-g-1", "groceries", "g1")
                            .color(colors.orange)
                            .stroke_width(2.0),
                        FlowEdge::new("e-g-2", "groceries", "g2")
                            .color(colors.orange)
                            .stroke_width(2.0),
                        FlowEdge::new("e-g-3", "groceries", "g3")
                            .color(colors.orange)
                            .stroke_width(2.0),
                        FlowEdge::new("e-g-4", "groceries", "g4")
                            .color(colors.orange)
                            .stroke_width(2.0),
                        FlowEdge::new("e-e-1", "errands", "e1")
                            .color(colors.blue)
                            .stroke_width(2.0),
                        FlowEdge::new("e-e-2", "errands", "e2")
                            .color(colors.blue)
                            .stroke_width(2.0),
                        FlowEdge::new("e-e-3", "errands", "e3")
                            .color(colors.blue)
                            .stroke_width(2.0),
                        FlowEdge::new("e-p-1", "projects", "p1")
                            .color(colors.green)
                            .stroke_width(2.0),
                        FlowEdge::new("e-p-2", "projects", "p2")
                            .color(colors.green)
                            .stroke_width(2.0),
                    ];

                    let state = cx.new(|_| FlowState::new(nodes, edges));

                    let flow = cx.new(|cx| {
                        FlowGraph::new(state.clone(), cx)
                            .no_node_chrome()
                            .bg_color(colors.bg)
                            .grid_color(colors.grid)
                            .node_renderer("root", render_root)
                            .node_renderer("branch", render_branch)
                            .node_renderer("leaf", render_leaf)
                    });

                    let minimap =
                        cx.new(|_| Minimap::new(state.clone()).container_bounds(1200.0, 800.0));

                    cx.new(|cx| MindMapApp {
                        flow,
                        state,
                        minimap,
                        focus_handle: cx.focus_handle(),
                        editing: None,
                        edit_buffer: String::new(),
                    })
                },
            )
            .expect("Failed to open window");
        });
}
