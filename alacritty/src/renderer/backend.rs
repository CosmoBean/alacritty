use crate::config::debug::RenderBackendPreference;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    OpenGl,
    Wgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendSelection {
    pub kind: BackendKind,
    pub wgpu_available: bool,
}

impl BackendSelection {
    pub fn is_fallback(&self, preference: RenderBackendPreference) -> bool {
        matches!((preference, self.kind), (RenderBackendPreference::Wgpu, BackendKind::OpenGl))
    }
}

pub fn select_backend(
    preference: RenderBackendPreference,
    wgpu_available: bool,
) -> BackendSelection {
    let kind = match preference {
        RenderBackendPreference::Auto => {
            if wgpu_available {
                BackendKind::Wgpu
            } else {
                BackendKind::OpenGl
            }
        },
        RenderBackendPreference::Wgpu => {
            if wgpu_available {
                BackendKind::Wgpu
            } else {
                BackendKind::OpenGl
            }
        },
        RenderBackendPreference::Gl => BackendKind::OpenGl,
    };

    BackendSelection { kind, wgpu_available }
}

#[cfg(test)]
mod tests {
    use super::{BackendKind, select_backend};
    use crate::config::debug::RenderBackendPreference;

    #[test]
    fn auto_prefers_wgpu_when_available() {
        let selected = select_backend(RenderBackendPreference::Auto, true);
        assert_eq!(selected.kind, BackendKind::Wgpu);
    }

    #[test]
    fn wgpu_falls_back_when_unavailable() {
        let selected = select_backend(RenderBackendPreference::Wgpu, false);
        assert_eq!(selected.kind, BackendKind::OpenGl);
    }
}
