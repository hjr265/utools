use gpui::{
    App, Application, ClickEvent, Context, Entity, Font, Menu, MenuItem, SharedString,
    Subscription, SystemMenuType, Window, WindowOptions, actions, div, font, prelude::*, px,
    relative, rgb,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, StyledExt, Theme, ThemeMode, h_flex,
    input::{InputEvent, InputState, TextInput},
    resizable::{ResizableState, h_resizable, resizable_panel},
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use utools::*;

struct Gallery {
    tools: Vec<(&'static str, Vec<Entity<ToolContainer>>)>,
    active_group_index: Option<usize>,
    active_index: Option<usize>,
    sidebar_collapsed: bool,
    search_input: Entity<InputState>,
    sidebar_state: Entity<ResizableState>,
    _subscriptions: Vec<Subscription>,
}

impl Gallery {
    pub fn new(init_tool: Option<&str>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let _subscriptions = vec![cx.subscribe(&search_input, |this, _, e, cx| match e {
            InputEvent::Change(_) => {
                this.active_group_index = Some(0);
                this.active_index = Some(0);
                cx.notify()
            }
            _ => {}
        })];

        let tools = vec![
            (
                "Base64",
                vec![
                    ToolContainer::panel::<Base64EncoderTool>(window, cx),
                    ToolContainer::panel::<Base64DecoderTool>(window, cx),
                ],
            ),
            (
                "JSON",
                vec![
                    ToolContainer::panel::<JSONFormatterTool>(window, cx),
                    ToolContainer::panel::<JSONViewerTool>(window, cx),
                ],
            ),
            (
                "Text",
                vec![ToolContainer::panel::<TextCharacterCountTool>(window, cx)],
            ),
        ];
        let mut this = Self {
            search_input,
            tools,
            active_group_index: Some(0),
            active_index: Some(0),
            sidebar_collapsed: false,
            sidebar_state: ResizableState::new(cx),
            _subscriptions,
        };

        if let Some(init_tool) = init_tool {
            this.set_active_tool(init_tool, window, cx);
        }

        this
    }

    fn set_active_tool(&mut self, name: &str, window: &mut Window, cx: &mut App) {
        let name = name.to_string();
        self.search_input.update(cx, |this, cx| {
            this.set_value(&name, window, cx);
        })
    }

    fn view(init_tool: Option<&str>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(init_tool, window, cx))
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.search_input.read(cx).value().trim().to_lowercase();

        let tools: Vec<_> = self
            .tools
            .iter()
            .filter_map(|(name, items)| {
                let filtered_items: Vec<_> = items
                    .iter()
                    .filter(|tool| tool.read(cx).name.to_lowercase().contains(&query))
                    .cloned()
                    .collect();
                if !filtered_items.is_empty() {
                    Some((name, filtered_items))
                } else {
                    None
                }
            })
            .collect();

        let active_group = self.active_group_index.and_then(|index| tools.get(index));
        let active_tool = self
            .active_index
            .and(active_group)
            .and_then(|group| group.1.get(self.active_index.unwrap()));
        let (tool_name, description) =
            if let Some(tool) = active_tool.as_ref().map(|tool| tool.read(cx)) {
                (tool.name.clone(), tool.description.clone())
            } else {
                ("".into(), "".into())
            };

        h_resizable("gallery-container", self.sidebar_state.clone())
            .child(
                resizable_panel()
                    .size(px(255.))
                    .size_range(px(200.)..px(320.))
                    .child(
                        Sidebar::left()
                            .width(relative(1.))
                            .border_width(px(0.))
                            .collapsed(self.sidebar_collapsed)
                            .header(
                                v_flex().w_full().gap_4().child(
                                    div()
                                        .bg(cx.theme().sidebar_accent)
                                        .px_1()
                                        .rounded_full()
                                        .flex_1()
                                        .mx_1()
                                        .child(
                                            TextInput::new(&self.search_input)
                                                .appearance(false)
                                                .cleanable(),
                                        ),
                                ),
                            )
                            .children(tools.clone().into_iter().enumerate().map(
                                |(group_ix, (group_name, sub_tools))| {
                                    SidebarGroup::new(*group_name).child(
                                        SidebarMenu::new().children(
                                            sub_tools.iter().enumerate().map(|(ix, tool)| {
                                                SidebarMenuItem::new(
                                                    tool.read(cx).short_name.clone(),
                                                )
                                                .active(
                                                    self.active_group_index == Some(group_ix)
                                                        && self.active_index == Some(ix),
                                                )
                                                .on_click(cx.listener(
                                                    move |this, _: &ClickEvent, _, cx| {
                                                        this.active_group_index = Some(group_ix);
                                                        this.active_index = Some(ix);
                                                        cx.notify();
                                                    },
                                                ))
                                            }),
                                        ),
                                    )
                                },
                            )),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .overflow_x_hidden()
                    .child(
                        h_flex()
                            .id("header")
                            .p_4()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .justify_between()
                            .items_start()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(div().text_xl().child(tool_name))
                                    .child(
                                        div()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(description),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("tool")
                            .flex_1()
                            .overflow_y_scroll()
                            .when_some(active_tool, |this, active_tool| {
                                this.child(active_tool.clone())
                            }),
                    )
                    .into_any_element(),
            )
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    // Parse `cargo run -- <tool_name>`
    let name = std::env::args().nth(1);

    app.run(|cx: &mut App| {
        gpui_component::init(cx);
        cx.activate(true);
        Theme::change(ThemeMode::Dark, None, cx);
        Theme::global_mut(cx).set_default_dark();
        Theme::global_mut(cx).font_family = "Space Grotesk".into();
        Theme::global_mut(cx).font_size = px(17.);
        utools::create_new_window(
            "μTools",
            move |window, cx| Gallery::view(name.as_deref(), window, cx),
            cx,
        );
    });
}

// Associate actions using the `actions!` macro (or `Action` derive macro)
actions!(set_menus, [Quit]);

// Define the quit function that is registered with the App
fn quit(_: &Quit, cx: &mut App) {
    println!("Gracefully quitting the application . . .");
    cx.quit();
}
