//! Virtual Xorg, desktop access and keyboard-assistance exercises.
type Result<T> = std::result::Result<T, String>;

pub mod remote;
pub mod session;
pub mod source;
pub mod typing_assist;
pub mod workspace;
pub mod xorg;
