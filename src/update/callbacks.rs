pub trait UpdateUi {
    fn on_progress(&self, _progress: f64) {}
    fn on_quit(&self);
}

pub struct NoopUi;

impl UpdateUi for NoopUi {
    fn on_quit(&self) {}
}
