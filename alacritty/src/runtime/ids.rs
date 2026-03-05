#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleKind {
    Window,
    Workspace,
    Surface,
}

pub fn window_ref(window_id: u64) -> String {
    format!("window:{window_id}")
}

pub fn workspace_ref(window_id: u64) -> String {
    format!("workspace:{window_id}")
}

pub fn surface_ref(window_id: u64) -> String {
    format!("surface:{window_id}")
}

pub fn parse_handle_ref(handle: &str) -> Option<(HandleKind, u64)> {
    let (kind, id) = handle.split_once(':')?;
    let id = id.parse().ok()?;
    let kind = match kind {
        "window" => HandleKind::Window,
        "workspace" => HandleKind::Workspace,
        "surface" => HandleKind::Surface,
        _ => return None,
    };

    Some((kind, id))
}

#[cfg(test)]
mod tests {
    use super::{HandleKind, parse_handle_ref};

    #[test]
    fn parse_valid_refs() {
        assert_eq!(parse_handle_ref("window:1"), Some((HandleKind::Window, 1)));
        assert_eq!(parse_handle_ref("workspace:2"), Some((HandleKind::Workspace, 2)));
        assert_eq!(parse_handle_ref("surface:3"), Some((HandleKind::Surface, 3)));
    }

    #[test]
    fn parse_invalid_refs() {
        assert_eq!(parse_handle_ref("foo"), None);
        assert_eq!(parse_handle_ref("window:x"), None);
        assert_eq!(parse_handle_ref("unknown:1"), None);
    }
}
