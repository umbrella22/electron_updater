use druid::Application;
use druid::{AppLauncher, Data, Lens, WindowDesc};

use crate::logging::{Log, Logger};
use crate::update::{run_task, UpdateUi};

mod widgets;

#[derive(Clone, Data, Lens, Default)]
pub struct UpdateState {
    pub progressbar: f64,
}

pub fn start_ui() {
    let main_window = WindowDesc::new(widgets::build_root_widget())
        .title("更新程序")
        .window_size((400.0, 40.0))
        .resizable(false)
        .show_titlebar(false);

    let initial_state: UpdateState = UpdateState { progressbar: 0.0 };
    let launcher = AppLauncher::with_window(main_window);
    let event_sink = launcher.get_external_handle();
    
    std::thread::spawn(move || update(event_sink));
    
    launcher
        .launch(initial_state)
        .expect("Failed to launch application");
}

struct DruidUi {
    event_sink: druid::ExtEventSink,
}

impl UpdateUi for DruidUi {
    fn on_progress(&self, progress: f64) {
        let event_sink = self.event_sink.clone();
        event_sink.add_idle_callback(move |state: &mut UpdateState| {
            state.progressbar = progress;
        })
    }

    fn on_quit(&self) {
        let event_sink = self.event_sink.clone();
        Log::info("退出");
        event_sink.add_idle_callback(move |_: &mut UpdateState| {
            Log::info("ui退出");
            Application::global().quit();
        })
    }
}

fn update(event_sink: druid::ExtEventSink) {
    let ui = DruidUi { event_sink };
    run_task(ui);
}
