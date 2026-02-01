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

        // Theme Colors - Minimalist Dark
        let root_bg = rgb(0x09090b); // zinc-950

        let theme_color = match self.status {
            UpdateStatus::Completed => rgb(0x22c55e), // green-500
            UpdateStatus::Failed | UpdateStatus::Cancelled => rgb(0xef4444), // red-500
            _ => rgb(0x3b82f6),                       // blue-500
        };

        let status_text = match self.status {
            UpdateStatus::Downloading => "正在更新...",
            UpdateStatus::Completed => "更新完成",
            UpdateStatus::Failed => "更新失败",
            UpdateStatus::Cancelled => "已取消",
        };

        // Calculate progress bar width (300 - 48 padding = 252)
        let bar_width = 252.0 * progress;

        // Helper for opacity
        let icon_bg = Rgba {
            r: theme_color.r,
            g: theme_color.g,
            b: theme_color.b,
            a: 0.1,
        };

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
                // Top Section: Icon/Percentage
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        match self.status {
                            UpdateStatus::Downloading => {
                                div()
                                    .text_size(px(36.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme_color)
                                    .child(format!("{}%", percentage))
                            },
                            _ => {
                                div()
                                    .w(px(48.0))
                                    .h(px(48.0))
                                    .rounded_full()
                                    .bg(icon_bg)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_2xl()
                                            .text_color(theme_color)
                                            .child(match self.status {
                                                UpdateStatus::Completed => "✓",
                                                UpdateStatus::Failed => "!",
                                                UpdateStatus::Cancelled => "×",
                                                _ => "",
                                            })
                                    )
                            }
                        }
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xa1a1aa)) // zinc-400
                            .child(status_text)
                    )
            )
            // Middle Section: Progress Bar (Only when downloading)
            .child(
                if let UpdateStatus::Downloading = self.status {
                    div()
                        .w_full()
                        .h(px(4.0))
                        .bg(rgb(0x27272a)) // zinc-800
                        .rounded_full()
                        .child(
                            div()
                                .h_full()
                                .bg(theme_color)
                                .rounded_full()
                                .w(px(bar_width))
                        )
                } else {
                    div().w_full().h(px(4.0)) // Placeholder
                }
            )
            // Bottom Section: Action Button
            .child(
                self.render_action_button(cx, theme_color)
            )
    }
}

impl UpdateView {
    fn render_action_button(&self, cx: &mut Context<Self>, _color: Rgba) -> impl IntoElement {
        let (label, click_handler): (&str, Box<dyn Fn(&mut UpdateView, &mut Window, &mut App)>) =
            match self.status {
                UpdateStatus::Downloading => (
                    "取消",
                    Box::new(|view, _, _| {
                        view.status = UpdateStatus::Cancelled;
                    }),
                ),
                UpdateStatus::Completed => (
                    "立即重启",
                    Box::new(|view, _, _| {
                        let _ = view.retry_tx.try_send(UiMsg::Quit);
                    }),
                ),
                UpdateStatus::Failed | UpdateStatus::Cancelled => (
                    "重试",
                    Box::new(|view, _, _| {
                        view.status = UpdateStatus::Downloading;
                        view.progress = 0.0;
                        let _ = view.retry_tx.try_send(UiMsg::Retry);
                    }),
                ),
            };

        div()
            .cursor_pointer()
            .px(px(24.0))
            .py(px(8.0))
            .rounded_md()
            .bg(rgb(0x27272a)) // zinc-800
            .hover(|s| s.bg(rgb(0x3f3f46))) // zinc-700
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xffffff))
                    .child(label)
            )
            .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, window, cx| {
                click_handler(view, window, cx);
            }))
    }
}
