use serde::Serialize;

use crate::runtime::ids::{surface_ref, window_ref, workspace_ref};

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SystemTree {
    pub windows: Vec<WindowNode>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WindowNode {
    pub reference: String,
    pub id: u64,
    pub title: String,
    pub workspaces: Vec<WorkspaceNode>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceNode {
    pub reference: String,
    pub id: u64,
    pub unread_count: usize,
    pub surfaces: Vec<SurfaceNode>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SurfaceNode {
    pub reference: String,
    pub id: u64,
    pub kind: SurfaceKind,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceKind {
    Terminal,
    Webview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub id: u64,
    pub title: String,
    pub workspaces: Vec<WorkspaceSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub id: u64,
    pub unread_count: usize,
    pub surfaces: Vec<SurfaceSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceSnapshot {
    pub id: u64,
    pub kind: SurfaceKind,
}

pub fn build_system_tree<I>(windows: I) -> SystemTree
where
    I: IntoIterator<Item = WindowSnapshot>,
{
    let mut windows: Vec<WindowSnapshot> = windows.into_iter().collect();
    windows.sort_by_key(|window| window.id);

    let windows = windows
        .into_iter()
        .map(|window| {
            let mut workspaces: Vec<WorkspaceSnapshot> = window.workspaces;
            workspaces.sort_by_key(|workspace| workspace.id);
            let workspaces = workspaces
                .into_iter()
                .map(|workspace| {
                    let mut surfaces: Vec<SurfaceSnapshot> = workspace.surfaces;
                    surfaces.sort_by_key(|surface| surface.id);
                    let surfaces = surfaces
                        .into_iter()
                        .map(|surface| SurfaceNode {
                            reference: surface_ref(surface.id),
                            id: surface.id,
                            kind: surface.kind,
                        })
                        .collect();

                    WorkspaceNode {
                        reference: workspace_ref(workspace.id),
                        id: workspace.id,
                        unread_count: workspace.unread_count,
                        surfaces,
                    }
                })
                .collect();

            WindowNode {
                reference: window_ref(window.id),
                id: window.id,
                title: window.title,
                workspaces,
            }
        })
        .collect();

    SystemTree { windows }
}

#[cfg(test)]
mod tests {
    use super::{
        SurfaceKind, SurfaceSnapshot, WindowSnapshot, WorkspaceSnapshot, build_system_tree,
    };

    #[test]
    fn build_sorted_tree() {
        let tree = build_system_tree([
            WindowSnapshot {
                id: 3,
                title: String::from("c"),
                workspaces: vec![WorkspaceSnapshot {
                    id: 3,
                    unread_count: 2,
                    surfaces: vec![SurfaceSnapshot { id: 3, kind: SurfaceKind::Terminal }],
                }],
            },
            WindowSnapshot {
                id: 1,
                title: String::from("a"),
                workspaces: vec![WorkspaceSnapshot {
                    id: 1,
                    unread_count: 0,
                    surfaces: vec![SurfaceSnapshot { id: 1, kind: SurfaceKind::Terminal }],
                }],
            },
        ]);

        assert_eq!(tree.windows[0].reference, "window:1");
        assert_eq!(tree.windows[0].workspaces[0].reference, "workspace:1");
        assert_eq!(tree.windows[1].reference, "window:3");
        assert_eq!(tree.windows[1].workspaces[0].unread_count, 2);
    }
}
