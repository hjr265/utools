use percent_encoding::percent_decode_str;

use gpui::{
    App, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    ParentElement, Render, SharedString, Styled, Window, div, px,
};

use gpui_component::{
    Disableable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    highlighter::Language,
    input::{InputState, TabSize, TextInput},
};

use crate::Tool;

pub struct URLDecoderTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    decoded: Entity<InputState>,
}

impl URLDecoderTool {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor(Language::Plain.name().to_string())
                .line_number(false)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .default_value("")
                .placeholder("Encoded URL")
        });
        let decoded = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor(Language::Plain.name().to_string())
                .line_number(false)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .default_value("")
                .placeholder("Decoded text")
        });

        Self {
            focus_handle: cx.focus_handle(),
            editor,
            decoded,
        }
    }

    fn on_decode_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.editor.read(cx).value().clone();
        self.decoded.update(cx, |state, cx| {
            let decoded_value = percent_decode_str(&value)
                .decode_utf8()
                .map(|s| s.to_string())
                .unwrap_or_default();
            state.set_value(SharedString::from(decoded_value), window, cx);
        })
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

    fn on_copy_decoded_click(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = self.decoded.read(cx).value().clone();
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
    }

    fn on_paste_decoded_click(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let value = clipboard.text().unwrap_or_default();
            self.decoded.update(cx, |state, cx| {
                state.set_value(value, window, cx);
            })
        }
    }
}

impl Tool for URLDecoderTool {
    fn title() -> &'static str {
        "URL Decoder"
    }

    fn short_title() -> &'static str {
        "Decoder"
    }

    fn description() -> &'static str {
        "Decodes a URL percent-encoded string back into text."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for URLDecoderTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for URLDecoderTool {
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
                        Button::new("decode-button")
                            .label("Decode")
                            .primary()
                            .disabled(value.is_empty())
                            .on_click(cx.listener(Self::on_decode_click)),
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
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("copy-decoded-button")
                            .label("Copy")
                            .on_click(cx.listener(Self::on_copy_decoded_click))
                            .ml_auto(),
                    )
                    .child(
                        Button::new("paste-decoded-button")
                            .label("Paste")
                            .on_click(cx.listener(Self::on_paste_decoded_click)),
                    ),
            )
            .child(
                TextInput::new(&self.decoded)
                    .h_full()
                    .font_family("Space Mono")
                    .text_size(px(15.))
                    .focus_bordered(false),
            )
    }
}
