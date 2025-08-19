use gpui::{
    App, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
};

use gpui_component::{
    Disableable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    highlighter::Language,
    input::{InputState, TabSize, TextInput},
};

use crate::Tool;

pub struct JSONViewerTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    view_mode: bool,
}

impl JSONViewerTool {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor(Language::Json.name().to_string())
                .line_number(true)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .default_value("")
                .placeholder("JSON Source")
        });

        Self {
            focus_handle: cx.focus_handle(),
            editor,
            view_mode: false,
        }
    }

    fn on_view_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = !self.view_mode;
        cx.notify();
    }

    fn on_copy_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let value = self.editor.read(cx).value().clone();
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
    }

    fn on_paste_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let value = clipboard.text().unwrap_or_default();
            self.editor.update(cx, |state, cx| {
                state.set_value(value, window, cx);
            })
        }
    }
}

impl Tool for JSONViewerTool {
    fn title() -> &'static str {
        "JSON Viewer"
    }

    fn short_title() -> &'static str {
        "Viewer"
    }

    fn description() -> &'static str {
        "JSON Viewer"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for JSONViewerTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for JSONViewerTool {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let value = self.editor.read(cx).value();

        div()
            .v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("view-button")
                            .label(if self.view_mode { "Back" } else { "View" })
                            .primary()
                            .disabled(value.is_empty())
                            .on_click(cx.listener(Self::on_view_click)),
                    )
                    .child(
                        Button::new("copy-button")
                            .label("Copy")
                            .on_click(cx.listener(Self::on_copy_click))
                            .ml_auto(),
                    )
                    .child(
                        Button::new("paste-button")
                            .label("Paste")
                            .on_click(cx.listener(Self::on_paste_click)),
                    ),
            )
            .when(self.view_mode, |this| this)
            .when(!self.view_mode, |this| {
                this.child(
                    TextInput::new(&self.editor)
                        .h_full()
                        .font_family("Space Mono")
                        .text_size(px(15.))
                        .focus_bordered(false),
                )
            })
    }
}

// fn make_tree(value: SharedString) {}
