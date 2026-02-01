pub trait UpdateUi {
    fn on_progress(&self, _progress: f64) {}
    fn on_failed(&self) {}
    fn on_quit(&self);
}
