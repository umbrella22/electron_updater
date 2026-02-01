use async_channel::{Receiver, Sender};
use gpui::*;

use crate::update::{run_task, UpdateUi};

use super::view::{UpdateStatus, UpdateView, WINDOW_HEIGHT, WINDOW_WIDTH};
use super::UiMsg;

pub fn start_ui() {
    if std::env::var("exe_path")
        .ok()
        .is_none_or(|path| !std::path::Path::new(&path).is_absolute())
        && !std::path::Path::new(".running_status").exists()
    {
        return;
    }
    let app = Application::new();

    app.run(move |cx| {
        let (tx, rx) = async_channel::unbounded::<UiMsg>();
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            is_resizable: false,
            window_min_size: Some(size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT))),
            ..Default::default()
        };

        let _window_handle = cx
            .open_window(window_options, move |window, cx| {
                window.set_window_title("更新程序");
                let view = cx.new(|_cx| UpdateView {
                    progress: 0.0,
                    status: UpdateStatus::Downloading,
                    retry_tx: tx.clone(),
                });

                start_event_loop(view.clone(), rx.clone(), tx.clone(), cx);

                std::thread::spawn(move || run_task(GpuiUi { tx }));

                view
            })
            .expect("Failed to open window");
    });
}

#[cfg(feature = "demo")]
pub fn start_demo_ui() {
    let app = Application::new();

    app.run(move |cx| {
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            is_resizable: false,
            window_min_size: Some(size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT))),
            ..Default::default()
        };
        let (tx, _rx) = async_channel::unbounded::<UiMsg>();
        let _window_handle = cx
            .open_window(window_options, move |window, cx| {
                window.set_window_title("更新程序（演示模式）");
                cx.new(|_cx| UpdateView {
                    progress: 0.6,
                    status: UpdateStatus::Downloading,
                    retry_tx: tx.clone(),
                })
            })
            .expect("Failed to open window");
    });
}

fn start_event_loop(view: Entity<UpdateView>, rx: Receiver<UiMsg>, tx: Sender<UiMsg>, cx: &App) {
    cx.spawn(async move |cx| {
        while let Ok(msg) = rx.recv().await {
            match msg {
                UiMsg::Progress(progress) => {
                    let progress = progress.clamp(0.0, 1.0);
                    view.update(cx, |view, cx| {
                        if view.status == UpdateStatus::Downloading {
                            view.progress = progress;
                            if progress >= 1.0 {
                                view.status = UpdateStatus::Completed;
                            }
                            cx.notify();
                        }
                    })
                    .ok();
                }
                UiMsg::Failed => {
                    view.update(cx, |view, cx| {
                        view.status = UpdateStatus::Failed;
                        cx.notify();
                    })
                    .ok();
                }
                UiMsg::Retry => {
                    view.update(cx, |view, cx| {
                        view.progress = 0.0;
                        view.status = UpdateStatus::Downloading;
                        cx.notify();
                    })
                    .ok();
                    let retry_tx = tx.clone();
                    std::thread::spawn(move || run_task(GpuiUi { tx: retry_tx }));
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

    fn on_failed(&self) {
        let _ = self.tx.try_send(UiMsg::Failed);
    }

    fn on_quit(&self) {
        let _ = self.tx.try_send(UiMsg::Quit);
    }
}
