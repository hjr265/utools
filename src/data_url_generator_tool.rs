use base64::{Engine as _, engine::general_purpose};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use gpui::{
    Action, App, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, ParentElement, Render, SharedString, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use gpui_component::{
    Disableable, StyledExt,
    button::{Button, ButtonVariants, DropdownButton},
    h_flex,
    highlighter::Language,
    input::{InputState, TabSize, TextInput},
};

use serde::Deserialize;

use crate::Tool;

#[derive(Clone, PartialEq, Eq, Deserialize)]
enum Encoding {
    Base64,
    URL,
}

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = data_url_tools, no_json)]
pub struct SetEncoding(Encoding);

const DATA_URL_ENCODE_SET: &AsciiSet = &CONTROLS.add(b',').add(b'%').add(b'#');

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = data_url_tools, no_json)]
pub struct SetMimeTypeAutoDetect(bool);

pub struct DataURLGeneratorTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    generated: Entity<InputState>,
    encoding: Encoding,
    mime_type_auto_detect: bool,
    mime_type: Entity<InputState>,
}

impl DataURLGeneratorTool {
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
        let generated = cx.new(|cx| {
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
        let mime_type = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("text/plain")
                .placeholder("Mime Type")
        });

        Self {
            focus_handle: cx.focus_handle(),
            editor,
            generated,
            encoding: Encoding::Base64,
            mime_type_auto_detect: true,
            mime_type,
        }
    }

    fn on_generate_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.editor.read(cx).value().clone();
        let generated_value = match self.encoding {
            Encoding::Base64 => format!(
                "base64,{}",
                general_purpose::URL_SAFE.encode(value.to_string())
            ),
            Encoding::URL => utf8_percent_encode(value.as_ref(), DATA_URL_ENCODE_SET).to_string(),
        };
        let mime_type = self.mime_type.read(cx).value().clone();
        let mime_type_extra = if mime_type == "text/plain" {
            ";charset=utf-8"
        } else {
            ""
        };
        self.generated.update(cx, |state, cx| {
            state.set_value(
                SharedString::from(format!(
                    "data:{}{};{}",
                    mime_type, mime_type_extra, generated_value
                )),
                window,
                cx,
            );
        })
    }

    fn on_paste_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let value = clipboard.text().unwrap_or_default();
            self.editor.update(cx, |state, cx| {
                state.set_value(value, window, cx);
            })
        }
    }

    fn on_copy_generated_click(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = self.generated.read(cx).value().clone();
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
    }

    fn on_action_set_encoding(
        &mut self,
        action: &SetEncoding,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.encoding = action.0.clone();
        cx.notify();
    }

    fn on_action_set_mime_type_auto_detect(
        &mut self,
        action: &SetMimeTypeAutoDetect,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.mime_type_auto_detect = action.0;
        cx.notify();
    }
}

impl Tool for DataURLGeneratorTool {
    fn title() -> &'static str {
        "Data URL Generator"
    }

    fn short_title() -> &'static str {
        "Generator"
    }

    fn description() -> &'static str {
        "Converts text into a data URL."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for DataURLGeneratorTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DataURLGeneratorTool {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let value = self.editor.read(cx).value();
        let encoding = self.encoding.clone();
        let mime_type_auto_detect = self.mime_type_auto_detect;

        div()
            .on_action(cx.listener(Self::on_action_set_encoding))
            .on_action(cx.listener(Self::on_action_set_mime_type_auto_detect))
            .v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        DropdownButton::new("generate-dropdown-button")
                            .primary()
                            .button(
                                Button::new("generate-button")
                                    .label("Generate")
                                    .primary()
                                    .disabled(value.is_empty())
                                    .on_click(cx.listener(Self::on_generate_click)),
                            )
                            .popup_menu(move |this, _, _| {
                                this.label("Encoding Size")
                                    .menu_with_check(
                                        "Base64",
                                        encoding == Encoding::Base64,
                                        Box::new(SetEncoding(Encoding::Base64)),
                                    )
                                    .menu_with_check(
                                        "URL",
                                        encoding == Encoding::URL,
                                        Box::new(SetEncoding(Encoding::URL)),
                                    )
                                    .label("Mime Type")
                                    .menu_with_check(
                                        "Auto-detect",
                                        mime_type_auto_detect,
                                        Box::new(SetMimeTypeAutoDetect(true)),
                                    )
                                    .menu_with_check(
                                        "Specific",
                                        !mime_type_auto_detect,
                                        Box::new(SetMimeTypeAutoDetect(false)),
                                    )
                            }),
                    )
                    .child(
                        Button::new("paste-button")
                            .label("Paste")
                            .on_click(cx.listener(Self::on_paste_click))
                            .ml_auto(),
                    ),
            )
            .when(!mime_type_auto_detect, |this| {
                this.child(TextInput::new(&self.mime_type).focus_bordered(false))
            })
            .child(
                TextInput::new(&self.editor)
                    .h_full()
                    .font_family("Space Mono")
                    .text_size(px(15.))
                    .focus_bordered(false),
            )
            .child(
                h_flex().gap_2().child(
                    Button::new("copy-generated-button")
                        .label("Copy")
                        .on_click(cx.listener(Self::on_copy_generated_click))
                        .ml_auto(),
                ),
            )
            .child(
                TextInput::new(&self.generated)
                    .h_full()
                    .font_family("Space Mono")
                    .text_size(px(15.))
                    .focus_bordered(false),
            )
    }
}
