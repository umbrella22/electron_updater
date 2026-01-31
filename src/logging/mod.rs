pub trait Logger {
    fn setup_logging() {}
    fn info(_info: &str) {}
    fn debug(_debug: &str) {}
    fn warn(_warn: &str) {}
    fn error(_error: &str) {}
}

#[cfg(feature = "debug")]
mod debug;

#[cfg(feature = "debug")]
pub use debug::Log;

#[cfg(not(feature = "debug"))]
pub struct Log {}

#[cfg(not(feature = "debug"))]
impl Logger for Log {}
