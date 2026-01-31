use async_channel::{Receiver, Sender};
use gpui::*;

use crate::update::{run_task, UpdateUi};

enum UiMsg {
    Progress(f32),
    Quit,
}

pub fn start_ui() {
    let app = Application::new();

    app.run(move |cx| {
        let (tx, rx) = async_channel::unbounded::<UiMsg>();

        let _window_handle = cx
            .open_window(WindowOptions::default(), move |window, cx| {
                window.set_window_title("更新程序");
                let view = cx.new(|_cx| UpdateView { progress: 0.0 });

                start_event_loop(view.clone(), rx.clone(), cx);

                std::thread::spawn(move || run_task(GpuiUi { tx }));

                view
            })
            .expect("Failed to open window");
    });
}

fn start_event_loop(view: Entity<UpdateView>, rx: Receiver<UiMsg>, cx: &App) {
    cx.spawn(async move |cx| {
        while let Ok(msg) = rx.recv().await {
            match msg {
                UiMsg::Progress(progress) => {
                    let progress = progress.clamp(0.0, 1.0);
                    view.update(cx, |view, cx| {
                        view.progress = progress;
                        cx.notify();
                    })
                    .ok();
                }
                UiMsg::Quit => std::process::exit(0),
            }
        }

        Ok::<_, anyhow::Error>(())
    })
    .detach();
}

struct GpuiUi {
    tx: Sender<UiMsg>,
}

impl UpdateUi for GpuiUi {
    fn on_progress(&self, progress: f64) {
        let _ = self.tx.try_send(UiMsg::Progress(progress as f32));
    }

    fn on_quit(&self) {
        let _ = self.tx.try_send(UiMsg::Quit);
        std::process::exit(0);
    }
}

struct UpdateView {
    progress: f32,
}

impl Render for UpdateView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let progress = self.progress.clamp(0.0, 1.0);
        let width = 360.0 * progress;

        div().size_full().items_center().justify_center().child(
            div()
                .w(px(360.0))
                .h(px(20.0))
                .rounded_xs()
                .bg(rgb(0x00BABABA))
                .child(
                    div()
                        .w(px(width))
                        .h(px(20.0))
                        .rounded_xs()
                        .bg(rgb(0x002D7DFF)),
                ),
        )
    }
}
