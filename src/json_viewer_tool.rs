use gpui::{
    App, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    ParentElement, Render, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};

use gpui_component::{
    Disableable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    highlighter::Language,
    input::{InputState, TabSize, TextInput},
    ListItem, TreeItem, TreeState, tree,
};

use crate::Tool;

pub struct JSONViewerTool {
    focus_handle: FocusHandle,
    editor: Entity<InputState>,
    tree_state: Entity<TreeState>,
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
        let tree_state = cx.new(|cx| TreeState::new(cx));

        Self {
            focus_handle: cx.focus_handle(),
            editor,
            tree_state,
            view_mode: false,
        }
    }

    fn on_view_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.view_mode {
            let value = self.editor.read(cx).value().clone();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&value) {
                let items = json_to_tree_items(&json, "root");
                self.tree_state.update(cx, |state, cx| {
                    state.set_items(items, cx);
                });
            }
        }
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

fn json_to_tree_items(value: &serde_json::Value, key: &str) -> Vec<TreeItem> {
    match value {
        serde_json::Value::Object(map) => {
            let children: Vec<TreeItem> = map
                .iter()
                .map(|(k, v)| {
                    let id = format!("{}/{}", key, k);
                    match v {
                        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                            let label = format_node_label(k, v);
                            let child_items = json_to_tree_items(v, &id);
                            TreeItem::new(id, label)
                                .children(child_items)
                                .expanded(true)
                        }
                        _ => TreeItem::new(id, format!("{}: {}", k, format_value(v))),
                    }
                })
                .collect();
            children
        }
        serde_json::Value::Array(arr) => {
            let children: Vec<TreeItem> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let id = format!("{}[{}]", key, i);
                    match v {
                        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                            let label = format_node_label(&format!("[{}]", i), v);
                            let child_items = json_to_tree_items(v, &id);
                            TreeItem::new(id, label)
                                .children(child_items)
                                .expanded(true)
                        }
                        _ => TreeItem::new(id, format!("[{}]: {}", i, format_value(v))),
                    }
                })
                .collect();
            children
        }
        _ => vec![TreeItem::new(key.to_string(), format_value(value))],
    }
}

fn format_node_label(key: &str, value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => format!("{} {{{} keys}}", key, map.len()),
        serde_json::Value::Array(arr) => format!("{} [{} items]", key, arr.len()),
        _ => key.to_string(),
    }
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s),
        _ => value.to_string(),
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
        "Displays JSON as a collapsible tree."
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
            .when(self.view_mode, |this| {
                this.child(
                    tree(&self.tree_state, |_ix, entry, _selected, _window, _cx| {
                        ListItem::new(SharedString::from(entry.item().id.clone()))
                            .px(px(16.) * entry.depth() as f32)
                            .child(
                                div()
                                    .font_family("Space Mono")
                                    .text_size(px(15.))
                                    .child(entry.item().label.clone()),
                            )
                    })
                    .h_full(),
                )
            })
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
