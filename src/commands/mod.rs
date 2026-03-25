pub const HOOK_MARKER_START: &str = "# >>> git-recap >>>";
pub const HOOK_MARKER_END: &str = "# <<< git-recap <<<";

pub mod digest;
pub mod hook;
pub mod info;
pub mod status;
pub mod this;
pub mod touch;
