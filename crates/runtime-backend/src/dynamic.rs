// Optional dynamic backend loader (dlopen-based) placeholder.
pub struct DynamicBackendLoader;

impl DynamicBackendLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn is_supported(&self) -> bool {
        false
    }
}
