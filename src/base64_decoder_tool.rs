use base64::{Engine as _, engine::general_purpose};

use gpui::{
    App, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    ParentElement, Render, SharedString, Styled, Window, div, px,
};

use gpui_component::StyledExt;
use gpui_component::{
    Disableable, button::Button, h_flex, highlighter::Language, input::InputState, input::TabSize,
    input::TextInput,
};

use crate::Tool;

pub struct Base64DecoderTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    decoded: Entity<InputState>,
}

impl Base64DecoderTool {
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
                .placeholder("Encoded Text")
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
                .placeholder("Decoded Text")
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
            match general_purpose::STANDARD.decode(value.to_string()) {
                Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                    Ok(decoded_value) => {
                        state.set_value(SharedString::from(decoded_value), window, cx);
                    }
                    Err(_) => {}
                },
                Err(_) => {}
            }
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

    fn on_copy_encoded_click(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = self.decoded.read(cx).value().clone();
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
    }

    fn on_paste_encoded_click(
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

impl Tool for Base64DecoderTool {
    fn title() -> &'static str {
        "Base64 Decoder"
    }

    fn short_title() -> &'static str {
        "Decoder"
    }

    fn description() -> &'static str {
        "Converts Base64 encoded string into text."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for Base64DecoderTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Base64DecoderTool {
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
                        Button::new("copy-encoded-button")
                            .label("Copy")
                            .on_click(cx.listener(Self::on_copy_encoded_click))
                            .ml_auto(),
                    )
                    .child(
                        Button::new("paste-encoded-button")
                            .label("Paste")
                            .on_click(cx.listener(Self::on_paste_encoded_click)),
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
