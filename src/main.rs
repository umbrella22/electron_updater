// 关闭window子系统
#![windows_subsystem = "windows"]

fn main() {
    #[cfg(feature = "druid")]
    updater::ui::start_ui();
    #[cfg(not(feature = "druid"))]
    struct HeadlessUi;
    #[cfg(not(feature = "druid"))]
    impl updater::task::UpdateUi for HeadlessUi {
        fn on_progress(&self, _progress: f64) {}
        fn on_quit(&self) {
            std::process::exit(0);
        }
    }
    #[cfg(not(feature = "druid"))]
    updater::task::run_task(HeadlessUi);
}
