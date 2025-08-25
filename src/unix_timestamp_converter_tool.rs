use chrono::{Datelike, Duration, Local, TimeZone, Utc};

use gpui::{
    App, AppContext, ClickEvent, Context, Entity, FocusHandle, Focusable, ParentElement, Render,
    Styled, Window, div, px,
};

use gpui_component::{
    Disableable, StyledExt,
    button::{Button, ButtonVariants},
    clipboard::Clipboard,
    h_flex,
    input::{InputState, TextInput},
    label::Label,
};

use crate::Tool;

pub struct UnixTimestampConverterTool {
    focus_handle: FocusHandle,
    input: Entity<InputState>,
    converted_utc: Entity<InputState>,
    converted_local: Entity<InputState>,
    since_relative: Entity<InputState>,
    days_since_epoch: Entity<InputState>,
    months_since_epoch: Entity<InputState>,
    day_of_year: Entity<InputState>,
}

impl UnixTimestampConverterTool {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Unix Timestamp"));
        let converted_utc = cx.new(|cx| InputState::new(window, cx));
        let converted_local = cx.new(|cx| InputState::new(window, cx));
        let since_relative = cx.new(|cx| InputState::new(window, cx));
        let days_since_epoch = cx.new(|cx| InputState::new(window, cx));
        let months_since_epoch = cx.new(|cx| InputState::new(window, cx));
        let day_of_year = cx.new(|cx| InputState::new(window, cx));

        Self {
            focus_handle: cx.focus_handle(),
            input,
            converted_utc,
            converted_local,
            since_relative,
            days_since_epoch,
            months_since_epoch,
            day_of_year,
        }
    }

    fn on_convert_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let value: i64 = self.input.read(cx).value().clone().parse().unwrap();
        let converted_utc = Utc.timestamp_opt(value, 0).unwrap();
        let converted_local = converted_utc.with_timezone(&Local);
        let now = Utc::now();
        let since_relative = format_relative_time(now.signed_duration_since(converted_utc));
        let epoch = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
        let days_since_epoch = (converted_utc - epoch).num_days();
        let months_since_epoch = (converted_utc.year() - epoch.year()) * 12
            + ((converted_utc.month() - epoch.month()) as i32);
        let day_of_year = converted_utc.ordinal();

        self.converted_utc.update(cx, |state, cx| {
            state.set_value(format!("{}", converted_utc), window, cx);
        });
        self.converted_local.update(cx, |state, cx| {
            state.set_value(format!("{}", converted_local), window, cx);
        });
        self.since_relative.update(cx, |state, cx| {
            state.set_value(since_relative, window, cx);
        });
        self.days_since_epoch.update(cx, |state, cx| {
            state.set_value(format!("{}", days_since_epoch), window, cx);
        });
        self.months_since_epoch.update(cx, |state, cx| {
            state.set_value(format!("{}", months_since_epoch), window, cx);
        });
        self.day_of_year.update(cx, |state, cx| {
            state.set_value(format!("{}", day_of_year), window, cx);
        });
    }

    fn on_now_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let value = Utc::now().timestamp();
        self.input.update(cx, |state, cx| {
            state.set_value(format!("{}", value), window, cx);
        })
    }

    fn on_paste_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let value = clipboard.text().unwrap_or_default();
            self.input.update(cx, |state, cx| {
                state.set_value(value, window, cx);
            })
        }
    }
}

impl Tool for UnixTimestampConverterTool {
    fn title() -> &'static str {
        "Unix Timestamp Converter"
    }

    fn short_title() -> &'static str {
        "Unix Timestamp Converter"
    }

    fn description() -> &'static str {
        "Transforms Unix timestamps into human-readable date and time formats."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for UnixTimestampConverterTool {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for UnixTimestampConverterTool {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let value = self.input.read(cx).value();
        let converted_utc = self.converted_utc.read(cx).value().clone();
        let converted_local = self.converted_utc.read(cx).value().clone();
        let converted_relative = self.since_relative.read(cx).value().clone();
        let days_since_epoch = self.days_since_epoch.read(cx).value().clone();
        let months_since_epoch = self.months_since_epoch.read(cx).value().clone();
        let day_of_year = self.day_of_year.read(cx).value().clone();

        div()
            .v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("convert-button")
                            .label("Convert")
                            .primary()
                            .disabled(value.is_empty())
                            .on_click(cx.listener(Self::on_convert_click)),
                    )
                    .child(
                        Button::new("now-button")
                            .label("Now")
                            .on_click(cx.listener(Self::on_now_click))
                            .ml_auto(),
                    )
                    .child(
                        Button::new("paste-button")
                            .label("Paste")
                            .on_click(cx.listener(Self::on_paste_click)),
                    ),
            )
            .child(
                TextInput::new(&self.input)
                    .text_size(px(15.))
                    .focus_bordered(false),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .mt_4()
                    .items_start()
                    .child(
                        div()
                            .v_flex()
                            .w_full()
                            .gap_2()
                            .child(Label::new("UTC"))
                            .child(
                                TextInput::new(&self.converted_utc)
                                    .text_size(px(15.))
                                    .focus_bordered(false)
                                    .suffix(
                                        Clipboard::new("converted-utc-clipboard")
                                            .value_fn(move |_, _| converted_utc.clone()),
                                    ),
                            )
                            .child(Label::new("Local"))
                            .child(
                                TextInput::new(&self.converted_local)
                                    .text_size(px(15.))
                                    .focus_bordered(false)
                                    .suffix(
                                        Clipboard::new("converted-local-clipboard")
                                            .value_fn(move |_, _| converted_local.clone()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .v_flex()
                            .w_full()
                            .gap_2()
                            .child(Label::new("Relative"))
                            .child(
                                TextInput::new(&self.since_relative)
                                    .text_size(px(15.))
                                    .focus_bordered(false)
                                    .suffix(
                                        Clipboard::new("converted-relative-clipboard")
                                            .value_fn(move |_, _| converted_relative.clone()),
                                    ),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .items_start()
                    .gap_2()
                    .mt_4()
                    .child(
                        div()
                            .v_flex()
                            .size_full()
                            .gap_2()
                            .child(Label::new("Days Since Epoch"))
                            .child(
                                TextInput::new(&self.days_since_epoch)
                                    .text_size(px(15.))
                                    .focus_bordered(false)
                                    .suffix(
                                        Clipboard::new("days-since-epoch-clipboard")
                                            .value_fn(move |_, _| days_since_epoch.clone()),
                                    ),
                            )
                            .child(Label::new("Months Since Epoch"))
                            .child(
                                TextInput::new(&self.months_since_epoch)
                                    .text_size(px(15.))
                                    .focus_bordered(false)
                                    .suffix(
                                        Clipboard::new("months-since-epoch-clipboard")
                                            .value_fn(move |_, _| months_since_epoch.clone()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .v_flex()
                            .size_full()
                            .gap_2()
                            .child(Label::new("Day of Year"))
                            .child(
                                TextInput::new(&self.day_of_year)
                                    .text_size(px(15.))
                                    .focus_bordered(false)
                                    .suffix(
                                        Clipboard::new("day-of-year-clipboard")
                                            .value_fn(move |_, _| day_of_year.clone()),
                                    ),
                            ),
                    ),
            )
    }
}

fn format_relative_time(duration: Duration) -> String {
    if duration.num_seconds() < 0 {
        let future_seconds = -duration.num_seconds();
        return format!("in {} seconds", future_seconds);
    }

    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();

    if days > 0 {
        return format!("{} days ago", days);
    } else if hours > 0 {
        return format!("{} hours ago", hours);
    } else if minutes > 0 {
        return format!("{} minutes ago", minutes);
    } else {
        return format!("{} seconds ago", seconds);
    }
}
