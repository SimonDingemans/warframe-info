use std::{future::Future, pin::Pin};

pub type DisplayResult<T> = Result<T, String>;
pub type DisplayOutputsFuture<'a> =
    Pin<Box<dyn Future<Output = DisplayResult<Vec<DisplayOutput>>> + Send + 'a>>;
pub type DynDisplayBackend = Box<dyn DisplayBackend>;

pub trait DisplayBackend: Send + Sync {
    fn display_outputs(&self) -> DisplayOutputsFuture<'_>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayOutput {
    pub name: Option<String>,
    pub position: (i32, i32),
    pub size: (u32, u32),
}

impl DisplayOutput {
    pub fn matches_name(&self, target: &str) -> bool {
        self.name
            .as_deref()
            .map(|name| name.eq_ignore_ascii_case(target.trim()))
            .unwrap_or(false)
    }
}

pub async fn display_outputs() -> DisplayResult<Vec<DisplayOutput>> {
    crate::platform::default_display_backend()
        .display_outputs()
        .await
}

pub fn reset_display_restore_token() -> DisplayResult<()> {
    crate::platform::reset_display_restore_token()
}
