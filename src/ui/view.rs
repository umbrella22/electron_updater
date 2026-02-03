use async_channel::Sender;
use gpui::*;

use super::UiMsg;

pub(crate) const WINDOW_WIDTH: f32 = 360.0;
pub(crate) const WINDOW_HEIGHT: f32 = 220.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdateStatus {
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

pub(crate) struct UpdateView {
    pub(crate) progress: f32,
    pub(crate) status: UpdateStatus,
    pub(crate) retry_tx: Sender<UiMsg>,
}

impl Render for UpdateView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let progress = self.progress.clamp(0.0, 1.0);
        let percentage = (progress * 100.0).round() as i32;

        // Colors from React version
        let slate_900 = rgb(0x0f172a);
        let white_10 = rgba(0xffffff1a);
        let white_20 = rgba(0xffffff33);
        let white_60 = rgba(0xffffff99);
        let blue_400 = rgb(0x60a5fa);
        let blue_500 = rgb(0x3b82f6);
        let green_400 = rgb(0x4ade80);
        let green_500 = rgb(0x22c55e);

        let status_text = match self.status {
            UpdateStatus::Downloading => "正在更新...",
            UpdateStatus::Completed => "更新完成",
            UpdateStatus::Failed => "更新失败",
            UpdateStatus::Cancelled => "已取消",
        };

        let status_color = match self.status {
            UpdateStatus::Completed => green_400,
            _ => blue_400,
        };

        div()
            .size_full()
            .bg(slate_900)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(360.0))
                    .bg(white_10)
                    .rounded_xl()
                    .border_1()
                    .border_color(white_20)
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .items_center()
                    .p(px(20.0))
                    .child(
                        // Progress Button Placeholder (Circular Progress)
                        div().mb(px(8.0)).child(self.render_progress_button(cx)),
                    )
                    .child(
                        // Title
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0xffffff))
                            .mb(px(12.0))
                            .child(status_text),
                    )
                    .child(
                        // Progress Section
                        div()
                            .w_full()
                            .mb(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .mb(px(8.0))
                                    .child(div().text_xs().text_color(white_60).child("进度"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(status_color)
                                            .child(format!("{}%", percentage)),
                                    ),
                            )
                            .child(
                                // Progress Bar
                                div()
                                    .relative()
                                    .h(px(8.0))
                                    .w_full()
                                    .bg(white_10)
                                    .rounded_full()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .bg(if self.status == UpdateStatus::Completed {
                                                green_500
                                            } else {
                                                blue_500
                                            })
                                            .rounded_full()
                                            .w(relative(progress)),
                                    ),
                            )
                            .child(
                                // Progress Details
                                div()
                                    .flex()
                                    .justify_end()
                                    .mt(px(8.0))
                                    .text_xs()
                                    .text_color(rgba(0xffffff66))
                                    .child(match self.status {
                                        UpdateStatus::Downloading => "约剩 30 秒",
                                        UpdateStatus::Cancelled | UpdateStatus::Failed => "已取消",
                                        _ => "",
                                    }),
                            ),
                    )
                    .child(
                        // Action Buttons
                        self.render_action_button(cx),
                    ),
            )
    }
}

impl UpdateView {
    fn render_progress_button(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let _is_active = self.status == UpdateStatus::Downloading;
        let is_completed = self.status == UpdateStatus::Completed;
        let is_failed =
            self.status == UpdateStatus::Failed || self.status == UpdateStatus::Cancelled;

        let color = if is_completed {
            rgb(0x22c55e)
        } else if is_failed {
            rgb(0xef4444)
        } else {
            rgb(0x60a5fa)
        };

        div()
            .relative()
            .size(px(48.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                // Outer ring (simplified as a border for now)
                div()
                    .absolute()
                    .size_full()
                    .rounded_full()
                    .border_2()
                    .border_color(color),
            )
            .child(
                // Inner circle with icon
                div()
                    .size(px(28.0))
                    .rounded_full()
                    .bg(if is_failed { rgba(0xef444433) } else { color })
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(match self.status {
                        UpdateStatus::Completed => "✓",
                        UpdateStatus::Failed | UpdateStatus::Cancelled => "✕",
                        _ => "↓",
                    }),
            )
    }

    fn render_action_button(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let retry_tx = self.retry_tx.clone();

        match self.status {
            UpdateStatus::Downloading => div(),
            UpdateStatus::Completed => div()
                .w_full()
                .py(px(6.0))
                .rounded_lg()
                .bg(rgb(0x22c55e))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, _| {
                    let _ = retry_tx.try_send(UiMsg::Quit);
                })
                .child(div().text_sm().text_color(rgb(0xffffff)).child("完成")),
            _ => div()
                .w_full()
                .py(px(6.0))
                .rounded_lg()
                .bg(rgb(0x3b82f6))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, _| {
                    let _ = retry_tx.try_send(UiMsg::Retry);
                })
                .child(div().text_sm().text_color(rgb(0xffffff)).child("重试")),
        }
    }
}
