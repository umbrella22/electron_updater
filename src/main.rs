// 关闭window子系统
#![windows_subsystem = "windows"]

fn main() {
    #[cfg(feature = "gpui")]
    updater::ui::start_ui();

    #[cfg(not(feature = "gpui"))]
    {
        struct HeadlessUi;
        impl updater::UpdateUi for HeadlessUi {
            fn on_progress(&self, _progress: f64) {}
            fn on_quit(&self) {
                std::process::exit(0);
            }
        }
        updater::run_task(HeadlessUi);
    }
}
