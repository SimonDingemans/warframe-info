#[derive(Debug, Clone, Default)]
pub struct UnsupportedDisplayBackend;

impl crate::DisplayBackend for UnsupportedDisplayBackend {
    fn display_outputs(&self) -> crate::DisplayOutputsFuture<'_> {
        Box::pin(async {
            Err("overlay display selection is not supported on this platform".to_owned())
        })
    }
}
