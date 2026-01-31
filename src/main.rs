// 关闭window子系统
#![windows_subsystem = "windows"]

fn main() {
    #[cfg(feature = "demo")]
    updater::ui::start_demo_ui();

    #[cfg(all(feature = "gpui", not(feature = "demo")))]
    updater::ui::start_ui();

    #[cfg(not(any(feature = "gpui", feature = "demo")))]
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
