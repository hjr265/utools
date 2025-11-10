mod assets;
mod base64_decoder_tool;
mod base64_encoder_tool;
mod data_url_generator_tool;
mod html_decoder_tool;
mod html_encoder_tool;
mod json_formatter_tool;
mod json_viewer_tool;
mod text_character_count_tool;
mod text_difference_tool;
mod title_bar;
mod unix_timestamp_converter_tool;

pub use assets::Assets;
use gpui::{
    AnyView, App, AppContext, Bounds, Context, Entity, Focusable, InteractiveElement, IntoElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window, WindowBounds, WindowKind,
    WindowOptions, div, prelude::*, px, size,
};

use gpui_component::{ActiveTheme, Root, TitleBar, v_flex};

pub use base64_decoder_tool::Base64DecoderTool;
pub use base64_encoder_tool::Base64EncoderTool;
pub use data_url_generator_tool::DataURLGeneratorTool;
pub use html_decoder_tool::HTMLDecoderTool;
pub use html_encoder_tool::HTMLEncoderTool;
pub use json_formatter_tool::JSONFormatterTool;
pub use json_viewer_tool::JSONViewerTool;
pub use text_character_count_tool::TextCharacterCountTool;
pub use text_difference_tool::TextDifferenceTool;
pub use title_bar::AppTitleBar;
pub use unix_timestamp_converter_tool::UnixTimestampConverterTool;

pub fn create_new_window<F, E>(title: &str, crate_view_fn: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    let mut window_size = size(px(1200.0), px(900.0));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }
    let window_bounds = Bounds::centered(None, window_size, cx);
    let title = SharedString::from(title.to_string());

    cx.spawn(async move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(640.),
                height: px(480.),
            }),
            kind: WindowKind::Normal,
            #[cfg(target_os = "linux")]
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            #[cfg(target_os = "linux")]
            window_decorations: Some(gpui::WindowDecorations::Client),
            ..Default::default()
        };

        let window = cx
            .open_window(options, |window, cx| {
                window.set_rem_size(cx.theme().font_size);

                let view = crate_view_fn(window, cx);
                let root = cx.new(|cx| ToolRoot::new(title.clone(), view, window, cx));

                cx.new(|cx| Root::new(root.into(), window, cx))
            })
            .expect("failed to open window");

        window
            .update(cx, |_, window, _| {
                window.activate_window();
                window.set_window_title(&title);
            })
            .expect("failed to update window");

        Ok::<_, anyhow::Error>(())
    })
    .detach();
}

struct ToolRoot {
    title_bar: Entity<AppTitleBar>,
    view: AnyView,
}

impl ToolRoot {
    pub fn new(
        title: impl Into<SharedString>,
        view: impl Into<AnyView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title_bar = cx.new(|cx| AppTitleBar::new(title, window, cx));
        Self {
            title_bar,
            view: view.into(),
        }
    }
}

impl Render for ToolRoot {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .font_family(cx.theme().font_family.clone())
            .size_full()
            .child(
                v_flex()
                    .size_full()
                    .child(self.title_bar.clone())
                    .child(div().flex_1().overflow_hidden().child(self.view.clone())),
            )
    }
}

pub trait Tool: Focusable + Render + Sized {
    fn klass() -> &'static str {
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    fn title() -> &'static str;
    fn short_title() -> &'static str;
    fn description() -> &'static str;

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable>;

    fn on_active(&mut self, active: bool, window: &mut Window, cx: &mut App) {
        let _ = active;
        let _ = window;
        let _ = cx;
    }
    fn on_active_any(view: AnyView, active: bool, window: &mut Window, cx: &mut App)
    where
        Self: 'static,
    {
        if let Some(tool) = view.downcast::<Self>().ok() {
            cx.update_entity(&tool, |tool, cx| {
                tool.on_active(active, window, cx);
            });
        }
    }
}

pub struct ToolContainer {
    focus_handle: gpui::FocusHandle,
    pub name: SharedString,
    pub short_name: SharedString,
    pub description: SharedString,
    tool: Option<AnyView>,
    tool_klass: Option<SharedString>,
    on_active: Option<fn(AnyView, bool, &mut Window, &mut App)>,
}

impl ToolContainer {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            name: "".into(),
            short_name: "".into(),
            description: "".into(),
            tool: None,
            tool_klass: None,
            on_active: None,
        }
    }

    pub fn panel<T: Tool>(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let name = T::title();
        let short_name = T::short_title();
        let description = T::description();
        let tool = T::new_view(window, cx);
        let tool_klass = T::klass();

        let view = cx.new(|cx| {
            let mut tool = Self::new(window, cx)
                .tool(tool.into(), tool_klass)
                .on_active(T::on_active_any);
            tool.name = name.into();
            tool.short_name = short_name.into();
            tool.description = description.into();
            tool
        });

        view
    }

    pub fn tool(mut self, tool: AnyView, tool_klass: impl Into<SharedString>) -> Self {
        self.tool = Some(tool);
        self.tool_klass = Some(tool_klass.into());
        self
    }

    pub fn on_active(mut self, on_active: fn(AnyView, bool, &mut Window, &mut App)) -> Self {
        self.on_active = Some(on_active);
        self
    }
}

impl Focusable for ToolContainer {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ToolContainer {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("tool-container")
            .size_full()
            .overflow_y_scroll()
            .track_focus(&self.focus_handle)
            // .on_action(cx.listener(Self::on_action_panel_info))
            // .on_action(cx.listener(Self::on_action_toggle_search))
            .when_some(self.tool.clone(), |this, tool| {
                this.child(
                    v_flex()
                        .id("tool-children")
                        .w_full()
                        .flex_1()
                        .p_4()
                        .child(tool),
                )
            })
    }
}
