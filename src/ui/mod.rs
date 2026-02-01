mod logic;
mod view;

pub(crate) enum UiMsg {
    Progress(f32),
    Failed,
    Retry,
    Quit,
}

pub use logic::start_ui;

#[cfg(feature = "demo")]
pub use logic::start_demo_ui;
