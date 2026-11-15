use gpui_flow::*;
use gpui_kit::gpui::*;

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub bg: gpui_kit::Hsla,
    pub grid: gpui_kit::Hsla,
    pub card: gpui_kit::Hsla,
    pub card_border: gpui_kit::Hsla,
    pub text: gpui_kit::Hsla,
    pub text_muted: gpui_kit::Hsla,
    pub accent_blue: gpui_kit::Hsla,
    pub accent_emerald: gpui_kit::Hsla,
    pub accent_violet: gpui_kit::Hsla,
    pub accent_amber: gpui_kit::Hsla,
    pub accent_rose: gpui_kit::Hsla,
}
const _: () = {
    impl Default for ColorScheme {
        fn default() -> Self {
            // Vercel/shadcn palette
            Self {
                bg: gpui_kit::rgb(0x09090b).into(),
                grid: gpui_kit::rgb(0x18181b).into(),
                card: gpui_kit::rgb(0x0a0a0c).into(),
                card_border: gpui_kit::rgb(0x27272a).into(),
                text: gpui_kit::rgb(0xfafafa).into(),
                text_muted: gpui_kit::rgb(0xa1a1aa).into(),
                accent_blue: gpui_kit::rgb(0x3b82f6).into(),
                accent_emerald: gpui_kit::rgb(0x10b981).into(),
                accent_violet: gpui_kit::rgb(0x8b5cf6).into(),
                accent_amber: gpui_kit::rgb(0xf59e0b).into(),
                accent_rose: gpui_kit::rgb(0xf43f5e).into(),
            }
        }
    }
};

type NodeDatum = ();
struct FlowExample {
    flow: Entity<FlowGraph<NodeDatum>>,
    minimap: Entity<Minimap<NodeDatum>>,
    controls: Entity<Controls<NodeDatum>>,
}

impl Render for FlowExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = ColorScheme::default();

        div()
            .size_full()
            .relative()
            .bg(colors.bg)
            .pt_20()
            .text_color(rgba(0xffffffff))
            .child("some-label")
            .child(self.flow.clone())
            // Controls bottom-left
            .child(
                div()
                    .absolute()
                    .bottom(px(16.0))
                    .left(px(16.0))
                    .child(self.controls.clone()),
            )
            // Minimap bottom-right
            .child(
                div()
                    .absolute()
                    .bottom(px(16.0))
                    .right(px(16.0))
                    .child(self.minimap.clone()),
            )
    }
}

fn node_text(node: &FlowNode<()>) -> String {
    if node.label.is_empty() {
        node.id.to_string()
    } else {
        node.label.to_string()
    }
}

/// Render node with a colored left accent bar.
fn render_node_with_accent(node: &FlowNode<()>, accent: Hsla, type_label: &str) -> AnyElement {
    let colors = ColorScheme::default();

    div()
        .flex()
        .flex_row()
        .child(
            // Accent bar
            div()
                .w(px(3.0))
                .rounded_l_sm()
                .bg(accent)
                .my(px(-8.0))
                .ml(px(-16.0))
                .mr(px(12.0)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted)
                        .font_weight(FontWeight::MEDIUM)
                        .child(type_label.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text)
                        .font_weight(FontWeight::MEDIUM)
                        .child(node_text(node)),
                ),
        )
        .into_any_element()
}

fn render_input(node: &FlowNode<NodeDatum>, _w: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();
    render_node_with_accent(node, colors.accent_emerald, "Input")
}

fn render_process(node: &FlowNode<NodeDatum>, _w: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();
    render_node_with_accent(node, colors.accent_blue, "Process")
}

fn render_output(node: &FlowNode<NodeDatum>, _w: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();
    render_node_with_accent(node, colors.accent_violet, "Output")
}

fn render_trigger(node: &FlowNode<NodeDatum>, _w: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();
    render_node_with_accent(node, colors.accent_amber, "Trigger")
}

fn render_sink(node: &FlowNode<NodeDatum>, _w: &mut Window, _cx: &mut App) -> AnyElement {
    let colors = ColorScheme::default();
    render_node_with_accent(node, colors.accent_rose, "Sink")
}

fn main() {
    gpui_kit::application()
        // Application::with_platform(Rc::new(gpui_macos::MacPlatform::new(false)))
        .run(move |cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(1100.0), px(750.0)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window, cx| {
                    let colors = ColorScheme::default();

                    let nodes = vec![
                        // Left column — triggers
                        FlowNode::new("webhook", 60.0, 80.0)
                            .label("Webhook")
                            .node_type("trigger")
                            .size(130.0, 52.0)
                            .handles(vec![HandleDef::source(HandlePosition::Right)]),
                        FlowNode::new("schedule", 60.0, 200.0)
                            .label("Cron Schedule")
                            .node_type("trigger")
                            .size(145.0, 52.0)
                            .handles(vec![HandleDef::source(HandlePosition::Right)]),
                        // Middle — processing
                        FlowNode::new("auth", 340.0, 80.0)
                            .label("Authenticate")
                            .node_type("process")
                            .size(140.0, 52.0)
                            .handles(vec![
                                HandleDef::target(HandlePosition::Left),
                                HandleDef::source(HandlePosition::Right),
                                HandleDef::source(HandlePosition::Bottom),
                            ]),
                        FlowNode::new("transform", 340.0, 200.0)
                            .label("Transform")
                            .node_type("process")
                            .size(125.0, 52.0)
                            .handles(vec![
                                HandleDef::target(HandlePosition::Left),
                                HandleDef::source(HandlePosition::Right),
                            ]),
                        FlowNode::new("validate", 340.0, 340.0)
                            .label("Validate")
                            .node_type("process")
                            .size(120.0, 52.0)
                            .handles(vec![
                                HandleDef::target(HandlePosition::Left),
                                HandleDef::source(HandlePosition::Right),
                            ]),
                        // Right column — data
                        FlowNode::new("db", 640.0, 80.0)
                            .label("Database")
                            .node_type("output")
                            .size(120.0, 52.0)
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("cache", 640.0, 200.0)
                            .label("Redis Cache")
                            .node_type("output")
                            .size(130.0, 52.0)
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        FlowNode::new("queue", 640.0, 340.0)
                            .label("Message Queue")
                            .node_type("sink")
                            .size(148.0, 52.0)
                            .handles(vec![HandleDef::target(HandlePosition::Left)]),
                        // Bottom — input
                        FlowNode::new("config", 60.0, 340.0)
                            .label("Environment")
                            .node_type("input")
                            .size(133.0, 52.0)
                            .handles(vec![HandleDef::source(HandlePosition::Right)]),
                    ];

                    let edges = vec![
                        FlowEdge::new("e1", "webhook", "auth")
                            .color(colors.accent_amber)
                            .stroke_width(2.0),
                        FlowEdge::new("e2", "schedule", "transform")
                            .color(colors.accent_amber)
                            .stroke_width(2.0),
                        FlowEdge::new("e3", "auth", "db")
                            .color(colors.accent_blue)
                            .stroke_width(2.0),
                        FlowEdge::new("e4", "auth", "transform")
                            .color(colors.accent_blue)
                            .stroke_width(1.5),
                        FlowEdge::new("e5", "transform", "cache")
                            .color(colors.accent_blue)
                            .stroke_width(2.0),
                        FlowEdge::new("e6", "config", "validate")
                            .color(colors.accent_emerald)
                            .stroke_width(2.0),
                        FlowEdge::new("e7", "validate", "queue")
                            .color(colors.accent_violet)
                            .stroke_width(2.0),
                    ];

                    let state = cx.new(|_| FlowState::new(nodes, edges));

                    let flow = cx.new(|cx| {
                        FlowGraph::new(state.clone(), cx)
                            .bg_color(colors.bg)
                            .grid_color(colors.grid)
                            .bg_pattern(BackgroundPattern::Cross)
                            .node_bg_color(colors.card)
                            .node_border_color(colors.card_border)
                            .node_renderer("input", render_input)
                            .node_renderer("process", render_process)
                            .node_renderer("output", render_output)
                            .node_renderer("trigger", render_trigger)
                            .node_renderer("sink", render_sink)
                    });

                    let minimap =
                        cx.new(|_| Minimap::new(state.clone()).container_bounds(1100.0, 750.0));
                    let controls = cx.new(|_| Controls::new(state).container_size(1100.0, 750.0));

                    cx.new(|_| FlowExample {
                        flow,
                        minimap,
                        controls,
                    })
                },
            )
            .expect("Failed to open window");
        });
}
