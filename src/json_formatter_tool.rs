use gpui::{
    Action, App, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, ParentElement, Render, SharedString, Styled, Window, div, px,
};

use gpui_component::StyledExt;
use gpui_component::{
    Disableable, button::Button, button::ButtonVariants, button::DropdownButton,
    dock::PanelControl, h_flex, highlighter::Language, input::InputState, input::TabSize,
    input::TextInput, popup_menu::PopupMenuExt, text::TextView, v_flex,
};

use serde::{Deserialize, Serialize};
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::{Value, json, to_writer_pretty};

use crate::Tool;

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = json_tools, no_json)]
pub struct SetIndentationSize(usize);

pub struct JSONFormatterTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    indent_size: usize,
}

impl JSONFormatterTool {
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
            indent_size: 2,
        }
    }

    fn on_format_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |state, cx| {
            let value = state.value();
            let json_value: Value = serde_json::from_str(value).unwrap();
            let indent = b" ".repeat(self.indent_size);
            let formatter = PrettyFormatter::with_indent(indent.as_slice());
            let mut writer = Vec::with_capacity(128);
            let mut serializer = Serializer::with_formatter(&mut writer, formatter);
            json_value.serialize(&mut serializer).unwrap();
            let pretty_json = String::from_utf8(writer).unwrap();
            state.set_value(SharedString::from(pretty_json), window, cx);
        })
    }

    fn on_compact_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |state, cx| {
            let value = state.value();
            let json_value: Value = serde_json::from_str(value).unwrap();
            let compact_json = serde_json::to_string(&json_value).unwrap();
            state.set_value(SharedString::from(compact_json), window, cx);
        })
    }

    fn on_copy_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
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

    fn on_action_set_indent_size(
        &mut self,
        action: &SetIndentationSize,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.indent_size = action.0;
        cx.notify();
    }
}

impl Tool for JSONFormatterTool {
    fn title() -> &'static str {
        "JSON Formatter"
    }

    fn short_title() -> &'static str {
        "Formatter"
    }

    fn description() -> &'static str {
        "Formats or compacts JSON data for better structure and clarity."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for JSONFormatterTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for JSONFormatterTool {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let value = self.editor.read(cx).value();
        let indentation_size = self.indent_size;

        div()
            .on_action(cx.listener(Self::on_action_set_indent_size))
            .v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        DropdownButton::new("format-dropdown-button")
                            .primary()
                            .button(
                                Button::new("format-button")
                                    .label("Format")
                                    .disabled(value.is_empty())
                                    .on_click(cx.listener(Self::on_format_click)),
                            )
                            .popup_menu(move |this, _, _| {
                                this.label("Indentation Size")
                                    .menu_with_check(
                                        "2",
                                        indentation_size == 2,
                                        Box::new(SetIndentationSize(2)),
                                    )
                                    .menu_with_check(
                                        "3",
                                        indentation_size == 3,
                                        Box::new(SetIndentationSize(3)),
                                    )
                                    .menu_with_check(
                                        "4",
                                        indentation_size == 4,
                                        Box::new(SetIndentationSize(4)),
                                    )
                            }),
                    )
                    .child(
                        Button::new("compact-button")
                            .label("Compact")
                            .disabled(value.is_empty())
                            .on_click(cx.listener(Self::on_compact_click)),
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
            .child(
                TextInput::new(&self.editor)
                    .h_full()
                    .font_family("Space Mono")
                    .text_size(px(15.))
                    .focus_bordered(false),
            )
    }
}
