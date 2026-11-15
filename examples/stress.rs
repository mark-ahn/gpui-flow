//! Stress test: 1000 nodes, incremental random edge additions.
//! Press Space to play/pause. Clean monochromatic dark UI.

use gpui_flow::*;
use gpui_kit::gpui::*;

const NODE_COUNT: usize = 1000;
const EDGES_PER_TICK: usize = 5;
const MAX_EDGES: usize = 200;
const TICK_MS: u64 = 50;

const COLS: usize = 40;
const SPACING_X: f32 = 260.0;
const SPACING_Y: f32 = 120.0;

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

struct StressApp {
    flow: Entity<FlowGraph<NodeDatum>>,
    state: Entity<FlowState<NodeDatum>>,
    minimap: Entity<Minimap<NodeDatum>>,
    controls: Entity<Controls<NodeDatum>>,
    focus_handle: FocusHandle,
    playing: bool,
    edges_added: usize,
    rng_state: u64,
}

impl StressApp {
    fn cheap_random(&mut self) -> u64 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        self.rng_state
    }

    fn add_random_edges(&mut self, cx: &mut Context<Self>) {
        if !self.playing || self.edges_added >= MAX_EDGES {
            return;
        }

        let _colors = ColorScheme::default();
        let count = EDGES_PER_TICK.min(MAX_EDGES - self.edges_added);
        let colors = [
            _colors.accent_blue,
            _colors.accent_emerald,
            _colors.accent_violet,
            _colors.accent_amber,
            _colors.accent_rose,
        ];

        let mut randoms = Vec::with_capacity(count * 3);
        for _ in 0..(count * 3) {
            randoms.push(self.cheap_random());
        }
        let edges_added_base = self.edges_added;

        self.state.update(cx, |state, _| {
            let node_count = state.nodes.len();
            if node_count < 2 {
                return;
            }
            for i in 0..count {
                let src_idx = (randoms[i * 3] as usize) % node_count;
                let tgt_idx = (randoms[i * 3 + 1] as usize) % node_count;
                if src_idx == tgt_idx {
                    continue;
                }
                let src_id = state.nodes[src_idx].id.clone();
                let tgt_id = state.nodes[tgt_idx].id.clone();
                let edge_id: SharedString = format!("se-{}", edges_added_base + i).into();
                let color = colors[(randoms[i * 3 + 2] as usize) % colors.len()];

                let edge = FlowEdge::new(edge_id, src_id, tgt_id)
                    .color(color)
                    .stroke_width(1.5);
                state.edges.push(edge);
            }
        });
        self.edges_added += count;
        cx.notify();
    }
}

impl Render for StressApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let node_count = state.nodes.len();
        let edge_count = state.edges.len();
        let _ = state;

        let status = if self.playing { "Playing" } else { "Paused" };
        let colors = ColorScheme::default();

        div()
            .id("stress-root")
            .track_focus(&self.focus_handle)
            .size_full()
            .relative()
            .bg(colors.bg)
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
            // Subtle status bar top-right
            .child(
                div()
                    .absolute()
                    .top(px(16.0))
                    .right(px(16.0))
                    .px_3()
                    .py_1p5()
                    .bg(gpui::rgba(0x18181b_ee))
                    .rounded_md()
                    .border_1()
                    .border_color(colors.card_border)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(format!(
                        "{} nodes  {}  edges  {}  +{}/{}",
                        node_count, "\u{00b7}", edge_count, self.edges_added, MAX_EDGES,
                    )),
            )
            // Status pill top-left
            .child(
                div()
                    .absolute()
                    .top(px(16.0))
                    .left(px(16.0))
                    .px_3()
                    .py_1p5()
                    .bg(gpui::rgba(0x18181b_ee))
                    .rounded_md()
                    .border_1()
                    .border_color(colors.card_border)
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w(px(6.0))
                            .h(px(6.0))
                            .rounded_full()
                            .bg(if self.playing {
                                colors.accent_emerald
                            } else {
                                colors.text_muted
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.text_muted)
                            .child(format!("{}  \u{00b7}  Space to toggle", status)),
                    ),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key: &str = event.keystroke.key.as_ref();
                if key == "space" {
                    this.playing = !this.playing;
                    if this.playing {
                        schedule_tick(cx);
                    }
                    cx.notify();
                }
            }))
    }
}

fn schedule_tick(cx: &mut Context<StressApp>) {
    cx.spawn(async move |this, cx| {
        cx.background_executor()
            .timer(std::time::Duration::from_millis(TICK_MS))
            .await;
        this.update(cx, |this, cx| {
            this.add_random_edges(cx);
            if this.playing && this.edges_added < MAX_EDGES {
                schedule_tick(cx);
            }
        })
        .ok();
    })
    .detach();
}

fn render_stress_node(
    node: &FlowNode<NodeDatum>,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    let colors = ColorScheme::default();
    div()
        .text_xs()
        .text_color(colors.text_muted)
        .child(if node.label.is_empty() {
            node.id.to_string()
        } else {
            node.label.to_string()
        })
        .into_any_element()
}

fn main() {
    gpui_kit::application()
        // Application::with_platform(Rc::new(gpui_macos::MacPlatform::new(false)))
        .run(move |cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(1400.0), px(900.0)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window, cx| {
                    let mut nodes = Vec::with_capacity(NODE_COUNT);
                    for i in 0..NODE_COUNT {
                        let col = i % COLS;
                        let row = i / COLS;
                        let x = col as f32 * SPACING_X;
                        let y = row as f32 * SPACING_Y;
                        let id: SharedString = format!("n{}", i).into();
                        nodes.push(
                            FlowNode::new(id, x, y)
                                .label(format!("#{}", i))
                                .node_type("stress")
                                .size(45.0, 24.0)
                                .handles(vec![
                                    HandleDef::target(HandlePosition::Left),
                                    HandleDef::source(HandlePosition::Right),
                                ]),
                        );
                    }

                    let colors = ColorScheme::default();
                    // Subtle grid-neighbor edges
                    let mut edges = Vec::new();
                    for i in 0..NODE_COUNT {
                        let col = i % COLS;
                        if col + 1 < COLS && i + 1 < NODE_COUNT {
                            let eid: SharedString = format!("ge{}", i).into();
                            edges.push(
                                FlowEdge::new(eid, format!("n{}", i), format!("n{}", i + 1))
                                    .color(colors.card_border)
                                    .stroke_width(1.0),
                            );
                        }
                    }

                    let state = cx.new(|_| FlowState::new(nodes, edges));

                    let flow = cx.new(|cx| {
                        FlowGraph::new(state.clone(), cx)
                            .bg_color(colors.bg)
                            .grid_color(colors.grid)
                            .bg_pattern(BackgroundPattern::Dots)
                            .node_bg_color(colors.card)
                            .node_border_color(colors.card_border)
                            .node_renderer("stress", render_stress_node)
                    });

                    let minimap =
                        cx.new(|_| Minimap::new(state.clone()).container_bounds(1400.0, 900.0));
                    let controls =
                        cx.new(|_| Controls::new(state.clone()).container_size(1400.0, 900.0));

                    cx.new(|cx| StressApp {
                        flow,
                        state,
                        minimap,
                        controls,
                        focus_handle: cx.focus_handle(),
                        playing: false,
                        edges_added: 0,
                        rng_state: 0xdeadbeef12345678,
                    })
                },
            )
            .expect("Failed to open window");
        });
}
