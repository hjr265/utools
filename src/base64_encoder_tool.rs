use base64::{Engine as _, engine::general_purpose};

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

pub struct Base64EncoderTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    encoded: Entity<InputState>,
}

impl Base64EncoderTool {
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
                .placeholder("Text")
        });
        let encoded = cx.new(|cx| {
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

        Self {
            focus_handle: cx.focus_handle(),
            editor,
            encoded,
        }
    }

    fn on_encode_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.editor.read(cx).value().clone();
        self.encoded.update(cx, |state, cx| {
            let encoded_value = general_purpose::STANDARD.encode(value.to_string());
            state.set_value(SharedString::from(encoded_value), window, cx);
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
        let value = self.encoded.read(cx).value().clone();
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
            self.encoded.update(cx, |state, cx| {
                state.set_value(value, window, cx);
            })
        }
    }
}

impl Tool for Base64EncoderTool {
    fn title() -> &'static str {
        "Base64 Encoder"
    }

    fn short_title() -> &'static str {
        "Encoder"
    }

    fn description() -> &'static str {
        "Converts text into a Base64 encoded string."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for Base64EncoderTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Base64EncoderTool {
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
                        Button::new("encode-button")
                            .label("Encode")
                            .primary()
                            .disabled(value.is_empty())
                            .on_click(cx.listener(Self::on_encode_click)),
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
                TextInput::new(&self.encoded)
                    .h_full()
                    .font_family("Space Mono")
                    .text_size(px(15.))
                    .focus_bordered(false),
            )
    }
}
