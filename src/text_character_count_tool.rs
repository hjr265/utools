use gpui::prelude::FluentBuilder;
use gpui::{
    Action, App, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, ParentElement, Render, SharedString, Styled, Window, div, px,
};

use gpui_component::{
    Disableable, button::Button, button::ButtonVariants, button::DropdownButton,
    clipboard::Clipboard, dock::PanelControl, h_flex, highlighter::Language, input::InputState,
    input::TabSize, input::TextInput, label::Label, popup_menu::PopupMenuExt, text::TextView,
    v_flex,
};

use serde::Deserialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::{Value, json};

use crate::Tool;

pub struct TextCharacterCountTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    character_count: usize,
}

impl TextCharacterCountTool {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .default_value("")
                .placeholder("Text")
        });

        Self {
            focus_handle: cx.focus_handle(),
            editor: editor,
            character_count: 0,
        }
    }

    fn on_count_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.editor.read(cx).value().clone();
        self.character_count = value.len();
        cx.notify();
    }

    fn on_copy_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.editor.read(cx).value().clone();
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        println!("{}", value.to_string());
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

impl Tool for TextCharacterCountTool {
    fn title() -> &'static str {
        "Text Character Count"
    }

    fn short_title() -> &'static str {
        "Character Count"
    }

    fn description() -> &'static str {
        "Counts characters in any text and display the total."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for TextCharacterCountTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextCharacterCountTool {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let character_count = self.character_count;

        v_flex()
            .size_full()
            .gap_1()
            .child(
                h_flex()
                    .gap_0p5()
                    .child(
                        Button::new("count-button")
                            .label("Count")
                            .on_click(cx.listener(Self::on_count_click)),
                    )
                    .child(div().px_4().when(character_count > 0, |this| {
                        this.child(
                            Clipboard::new("count-clipboard")
                                .content(move |_, _| {
                                    Label::new(format!("{} characters", character_count))
                                })
                                .value_fn({
                                    let view = cx.entity().clone();
                                    move |_, cx| SharedString::from(format!("{}", character_count))
                                }),
                        )
                    }))
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
                v_flex().id("source").w_full().flex_1().gap_2().child(
                    TextInput::new(&self.editor)
                        .bordered(false)
                        .h_full()
                        .font_family("Space Mono")
                        .text_size(px(15.))
                        .focus_bordered(false),
                ),
            )
    }
}
