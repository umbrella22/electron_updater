pub trait UpdateUi {
    fn on_progress(&self, _progress: f64) {}
    fn on_quit(&self);
}
