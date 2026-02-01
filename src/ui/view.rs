use async_channel::Sender;
use gpui::*;

use super::UiMsg;

pub(crate) const WINDOW_WIDTH: f32 = 300.0;
pub(crate) const WINDOW_HEIGHT: f32 = 180.0;

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

        let root_bg = rgb(0x09090b);

        let theme_color = match self.status {
            UpdateStatus::Completed => rgb(0x22c55e),
            UpdateStatus::Failed | UpdateStatus::Cancelled => rgb(0xef4444),
            _ => rgb(0x3b82f6),
        };

        let status_text = match self.status {
            UpdateStatus::Downloading => "正在更新...",
            UpdateStatus::Completed => "更新完成",
            UpdateStatus::Failed => "更新失败",
            UpdateStatus::Cancelled => "已取消",
        };

        let bar_width = 252.0 * progress;
        let is_downloading = matches!(self.status, UpdateStatus::Downloading);
        let primary_text = if is_downloading {
            format!("{}%", percentage)
        } else {
            status_text.to_string()
        };
        let primary_size = if is_downloading { px(36.0) } else { px(28.0) };

        div()
            .size_full()
            .bg(root_bg)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p(px(24.0))
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_size(primary_size)
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme_color)
                            .child(primary_text),
                    )
                    .child(if is_downloading {
                        div().text_sm().text_color(rgb(0xa1a1aa)).child(status_text)
                    } else {
                        div().h(px(0.0))
                    }),
            )
            .child(if let UpdateStatus::Downloading = self.status {
                div()
                    .w_full()
                    .h(px(4.0))
                    .bg(rgb(0x27272a))
                    .rounded_full()
                    .child(
                        div()
                            .h_full()
                            .bg(theme_color)
                            .rounded_full()
                            .w(px(bar_width)),
                    )
            } else {
                div().w_full().h(px(4.0))
            })
            .child(self.render_action_button(cx))
    }
}

impl UpdateView {
    fn render_action_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status;
        let label = match status {
            UpdateStatus::Downloading => "取消",
            UpdateStatus::Completed => "立即重启",
            UpdateStatus::Failed | UpdateStatus::Cancelled => "重试",
        };

        div()
            .cursor_pointer()
            .px(px(24.0))
            .py(px(8.0))
            .rounded_md()
            .bg(rgb(0x27272a))
            .hover(|s| s.bg(rgb(0x3f3f46)))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xffffff))
                    .child(label),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |view, _, _window, _cx| match status {
                    UpdateStatus::Downloading => {
                        view.status = UpdateStatus::Cancelled;
                    }
                    UpdateStatus::Completed => {
                        let _ = view.retry_tx.try_send(UiMsg::Quit);
                    }
                    UpdateStatus::Failed | UpdateStatus::Cancelled => {
                        view.status = UpdateStatus::Downloading;
                        view.progress = 0.0;
                        let _ = view.retry_tx.try_send(UiMsg::Retry);
                    }
                }),
            )
    }
}
