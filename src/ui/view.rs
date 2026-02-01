use async_channel::Sender;
use gpui::*;

use super::UiMsg;

const WINDOW_WIDTH: f32 = 360.0;
const WINDOW_HEIGHT: f32 = 233.0;
const CARD_PADDING_X: f32 = 20.0;
const CARD_PADDING_Y: f32 = 16.0;
const PROGRESS_BAR_WIDTH: f32 = 300.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdateStatus {
    Downloading,
    Completed,
    Failed,
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
        let progress_width = PROGRESS_BAR_WIDTH * progress;
        let status_text = match self.status {
            UpdateStatus::Downloading => "正在更新...",
            UpdateStatus::Completed => "更新完成",
            UpdateStatus::Failed => "更新失败",
        };
        let status_hint = match self.status {
            UpdateStatus::Downloading => "约剩 30 秒",
            UpdateStatus::Completed => "",
            UpdateStatus::Failed => "请重试",
        };
        let progress_color = match self.status {
            UpdateStatus::Completed => rgb(0x22c55e),
            UpdateStatus::Failed => rgb(0xef4444),
            UpdateStatus::Downloading => rgb(0x3b82f6),
        };
        let progress_track = rgb(0xffffff);
        let card_background = rgb(0x1f2937);
        let root_background = rgb(0x0f172a);

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(root_background)
            .child(
                div()
                    .w(px(WINDOW_WIDTH))
                    .h(px(WINDOW_HEIGHT))
                    .px(px(CARD_PADDING_X))
                    .py(px(CARD_PADDING_Y))
                    .bg(card_background)
                    .rounded_md()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .w(px(44.0))
                            .h(px(44.0))
                            .rounded_full()
                            .bg(progress_color)
                            .text_color(rgb(0xffffff))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xl()
                            .child(match self.status {
                                UpdateStatus::Downloading => "↓",
                                UpdateStatus::Completed => "✓",
                                UpdateStatus::Failed => "×",
                            }),
                    )
                    .child(div().text_color(rgb(0xffffff)).text_xl().child(status_text))
                    .child(
                        div()
                            .w(px(PROGRESS_BAR_WIDTH))
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_color(rgb(0x94a3b8))
                                    .text_sm()
                                    .child(format!("进度 {percentage}%")),
                            )
                            .child(
                                div()
                                    .w(px(PROGRESS_BAR_WIDTH))
                                    .h(px(8.0))
                                    .rounded_full()
                                    .bg(progress_track)
                                    .child(
                                        div()
                                            .w(px(progress_width))
                                            .h_full()
                                            .rounded_full()
                                            .bg(progress_color),
                                    ),
                            )
                            .child(
                                div()
                                    .h(px(12.0))
                                    .text_color(rgb(0x94a3b8))
                                    .text_sm()
                                    .child(status_hint),
                            ),
                    )
                    .child(match self.status {
                        UpdateStatus::Downloading => div().w(px(PROGRESS_BAR_WIDTH)).h(px(32.0)),
                        UpdateStatus::Completed => div()
                            .w(px(PROGRESS_BAR_WIDTH))
                            .h(px(32.0))
                            .rounded_md()
                            .bg(progress_color)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("完成"),
                        UpdateStatus::Failed => div()
                            .w(px(PROGRESS_BAR_WIDTH))
                            .h(px(32.0))
                            .rounded_md()
                            .bg(rgb(0x0ea5e9))
                            .hover(|s| s.bg(rgb(0x38bdf8)))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .cursor_pointer()
                            .child("重试")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|view, _, _, _| {
                                    let _ = view.retry_tx.try_send(UiMsg::Retry);
                                }),
                            ),
                    }),
            )
    }
}
