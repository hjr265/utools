use std::ops::Range;

use gpui::prelude::FluentBuilder;
use gpui::{
    Action, App, AppContext, Axis, ClickEvent, ClipboardItem, Context, Entity, FocusHandle,
    Focusable, HighlightStyle, Hsla, InteractiveElement, IntoElement, ParentElement, Render, Rgba,
    SharedString, Styled, StyledText, Subscription, Window, div, px, rems,
};

use gpui_component::button::DropdownButton;
use gpui_component::{ActiveTheme, Size, StyleSized, StyledExt};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    input::TextInput,
    scroll::ScrollbarAxis,
    v_flex,
};

use serde::Deserialize;
use similar::{ChangeTag, TextDiff};

use crate::Tool;

#[derive(Clone, PartialEq, Eq, Deserialize)]
enum Granularity {
    Character,
    Word,
    Line,
}

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = data_url_tools, no_json)]
pub struct SetGranularity(Granularity);

pub struct TextDifferenceTool {
    focus_handle: FocusHandle,
    original: Entity<InputState>,
    modified: Entity<InputState>,
    granularity: Granularity,
    difference_text: String,
    difference_highlights: Vec<(Range<usize>, HighlightStyle)>,
}

impl TextDifferenceTool {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let original = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .default_value("")
                .placeholder("Original")
        });
        let modified = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .default_value("")
                .placeholder("Modified")
        });

        Self {
            focus_handle: cx.focus_handle(),
            original,
            modified,
            granularity: Granularity::Word,
            difference_text: String::new(),
            difference_highlights: Vec::new(),
        }
    }

    fn on_compare_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let old = self.original.read(cx).value().clone();
        let new = self.modified.read(cx).value().clone();

        let diff = match self.granularity {
            Granularity::Character => TextDiff::from_chars(old.as_str(), new.as_str()),
            Granularity::Word => TextDiff::from_words(old.as_str(), new.as_str()),
            Granularity::Line => TextDiff::from_lines(old.as_str(), new.as_str()),
        };

        let colour_for = |tag: ChangeTag| -> Hsla {
            match tag {
                ChangeTag::Delete => cx.theme().red,
                ChangeTag::Insert => cx.theme().green,
                ChangeTag::Equal => cx.theme().foreground,
            }
        };

        let mut text = String::with_capacity(old.len() + new.len());
        let mut highlights = Vec::new();
        for op in diff.ops() {
            for change in diff.iter_changes(op) {
                let pos = text.len();
                text.push_str(change.value());
                highlights.push((
                    pos..text.len(),
                    HighlightStyle {
                        color: Some(colour_for(change.tag())),
                        ..Default::default()
                    },
                ));
            }
        }

        self.difference_text = text;
        self.difference_highlights = highlights;

        cx.notify();
    }

    fn on_back_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.difference_text = String::new();
        self.difference_highlights = Vec::new();
    }

    fn on_copy_original_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let value = self.original.read(cx).value().clone();
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
    }

    fn on_paste_original_click(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let value = clipboard.text().unwrap_or_default();
            self.original.update(cx, |state, cx| {
                state.set_value(value, window, cx);
            })
        }
    }

    fn on_copy_modified_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let value = self.modified.read(cx).value().clone();
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
    }

    fn on_paste_modified_click(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let value = clipboard.text().unwrap_or_default();
            self.modified.update(cx, |state, cx| {
                state.set_value(value, window, cx);
            })
        }
    }

    fn on_action_set_granularity(
        &mut self,
        action: &SetGranularity,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.granularity = action.0.clone();
        cx.notify();
    }
}

impl Tool for TextDifferenceTool {
    fn title() -> &'static str {
        "Text Difference"
    }

    fn short_title() -> &'static str {
        "Difference"
    }

    fn description() -> &'static str {
        "Shows differences between two texts."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for TextDifferenceTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextDifferenceTool {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let granularity = self.granularity.clone();
        let difference_text = self.difference_text.clone();
        let difference_highlights = self.difference_highlights.clone();

        div()
            .on_action(cx.listener(Self::on_action_set_granularity))
            .v_flex()
            .size_full()
            .gap_2()
            .when_else(
                self.difference_text == "",
                |this| {
                    this.child(
                        h_flex()
                            .gap_2()
                            .child(
                                DropdownButton::new("compare-dropdown-button")
                                    .primary()
                                    .button(
                                        Button::new("compare-button")
                                            .label("Compare")
                                            .primary()
                                            .on_click(cx.listener(Self::on_compare_click)),
                                    )
                                    .popup_menu(move |this, _, _| {
                                        this.label("Granularity")
                                            .menu_with_check(
                                                "Character",
                                                granularity == Granularity::Character,
                                                Box::new(SetGranularity(Granularity::Character)),
                                            )
                                            .menu_with_check(
                                                "Word",
                                                granularity == Granularity::Word,
                                                Box::new(SetGranularity(Granularity::Word)),
                                            )
                                            .menu_with_check(
                                                "Line",
                                                granularity == Granularity::Line,
                                                Box::new(SetGranularity(Granularity::Line)),
                                            )
                                    }),
                            )
                            .child(
                                Button::new("copy-original-button")
                                    .label("Copy")
                                    .on_click(cx.listener(Self::on_copy_original_click))
                                    .ml_auto(),
                            )
                            .child(
                                Button::new("paste-original-button")
                                    .label("Paste")
                                    .on_click(cx.listener(Self::on_paste_original_click)),
                            ),
                    )
                    .child(
                        v_flex().id("origin").w_full().flex_1().gap_2().child(
                            TextInput::new(&self.original)
                                .h_full()
                                .font_family("Space Mono")
                                .text_size(px(15.))
                                .focus_bordered(false),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("copy-modified-button")
                                    .label("Copy")
                                    .on_click(cx.listener(Self::on_copy_modified_click))
                                    .ml_auto(),
                            )
                            .child(
                                Button::new("paste-modified-button")
                                    .label("Paste")
                                    .on_click(cx.listener(Self::on_paste_modified_click)),
                            ),
                    )
                    .child(
                        v_flex().id("modified").w_full().flex_1().gap_2().child(
                            TextInput::new(&self.modified)
                                .h_full()
                                .font_family("Space Mono")
                                .text_size(px(15.))
                                .focus_bordered(false),
                        ),
                    )
                },
                |this| {
                    this.child(
                        h_flex().gap_2().child(
                            Button::new("back-button")
                                .label("Back")
                                .primary()
                                .on_click(cx.listener(Self::on_back_click)),
                        ),
                    )
                    .child(
                        h_flex().id("source").w_full().flex_1().gap_2().child(
                            div()
                                .size_full()
                                .font_family("Space Mono")
                                .text_size(px(15.))
                                .line_height(rems(1.25))
                                .bg(cx.theme().background)
                                .text_color(cx.theme().foreground)
                                .rounded(cx.theme().radius)
                                .border_color(cx.theme().input)
                                .border_1()
                                .input_px(Size::default())
                                .input_py(Size::default())
                                .when(cx.theme().shadow, |this| this.shadow_xs())
                                .child(
                                    StyledText::new(difference_text)
                                        .with_highlights(difference_highlights),
                                ),
                        ),
                    )
                },
            )
    }
}
