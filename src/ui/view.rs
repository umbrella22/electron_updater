use gpui::*;

pub(crate) struct UpdateView {
    pub(crate) progress: f32,
}

impl Render for UpdateView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let progress = self.progress.clamp(0.0, 1.0);
        let width = 360.0 * progress;
        let percentage = (progress * 100.0) as i32;

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(rgb(0xffffff))
            .text_color(rgb(0x333333))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(16.0))
                    .child(div().text_xl().child("正在更新..."))
                    .child(
                        // Progress Bar Container
                        div()
                            .w(px(360.0))
                            .h(px(8.0))
                            .rounded_full()
                            .bg(rgb(0xe5e7eb))
                            .child(
                                // Progress Fill
                                div().w(px(width)).h_full().rounded_full().bg(rgb(0x3b82f6)),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6b7280))
                            .child(format!("已完成 {}%", percentage)),
                    ),
            )
            .child(
                div().mt(px(32.0)).child(
                    div()
                        .px(px(16.0))
                        .py(px(8.0))
                        .bg(rgb(0xf3f4f6))
                        .hover(|s| s.bg(rgb(0xe5e7eb)))
                        .rounded_md()
                        .cursor_pointer()
                        .child("取消更新")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|_, _, _, _| {
                                std::process::exit(0);
                            }),
                        ),
                ),
            )
    }
}
