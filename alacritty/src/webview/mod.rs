use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebviewSurface {
    pub id: String,
    pub workspace_id: String,
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebviewFrame {
    pub width: u32,
    pub height: u32,
    pub generation: u64,
}

pub trait OffscreenWebview {
    fn navigate(&mut self, url: String);
    fn evaluate_javascript(&mut self, script: String);
    fn poll_frame(&mut self) -> Option<WebviewFrame>;
}

#[derive(Default, Debug)]
pub struct WebviewStore {
    next_id: u64,
    surfaces: Vec<WebviewSurface>,
}

impl WebviewStore {
    pub fn open(&mut self, workspace_id: String, url: String) -> &WebviewSurface {
        const WEBVIEW_SURFACE_ID_BASE: u64 = 10_000_000;

        let surface = WebviewSurface {
            id: format!("surface:{}", WEBVIEW_SURFACE_ID_BASE + self.next_id),
            workspace_id,
            url,
            title: None,
        };
        self.next_id += 1;
        self.surfaces.push(surface);

        self.surfaces.last().expect("webview insert")
    }

    pub fn navigate(&mut self, id: &str, url: String) -> Option<&WebviewSurface> {
        let surface = self.surfaces.iter_mut().find(|surface| surface.id == id)?;
        surface.url = url;
        Some(surface)
    }

    pub fn close(&mut self, id: &str) -> Option<WebviewSurface> {
        let index = self.surfaces.iter().position(|surface| surface.id == id)?;
        Some(self.surfaces.remove(index))
    }

    pub fn list(&self) -> &[WebviewSurface] {
        &self.surfaces
    }

    pub fn list_for_workspace(&self, workspace_id: &str) -> Vec<&WebviewSurface> {
        self.surfaces.iter().filter(|surface| surface.workspace_id == workspace_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::WebviewStore;

    #[test]
    fn open_and_navigate_and_close() {
        let mut store = WebviewStore::default();
        let opened =
            store.open(String::from("workspace:1"), String::from("https://example.com")).id.clone();
        assert_eq!(store.list().len(), 1);

        let _ = store.navigate(&opened, String::from("https://alacritty.org"));
        assert_eq!(store.list()[0].url, "https://alacritty.org");

        assert!(store.close(&opened).is_some());
        assert!(store.list().is_empty());
    }
}
