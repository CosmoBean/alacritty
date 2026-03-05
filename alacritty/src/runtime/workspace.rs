#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

impl SplitDirection {
    pub fn orientation(self) -> SplitOrientation {
        match self {
            SplitDirection::Left | SplitDirection::Right => SplitOrientation::Horizontal,
            SplitDirection::Up | SplitDirection::Down => SplitOrientation::Vertical,
        }
    }

    pub fn insert_first(self) -> bool {
        matches!(self, SplitDirection::Left | SplitDirection::Up)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneNode {
    Leaf(String),
    Split { orientation: SplitOrientation, first: Box<PaneNode>, second: Box<PaneNode> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneTree {
    pub root: PaneNode,
}

impl PaneTree {
    pub fn new(initial_surface: String) -> Self {
        Self { root: PaneNode::Leaf(initial_surface) }
    }

    pub fn split(
        &mut self,
        target_surface: &str,
        direction: SplitDirection,
        new_surface: String,
    ) -> bool {
        split_node(&mut self.root, target_surface, direction, new_surface)
    }

    pub fn surfaces(&self) -> Vec<&str> {
        let mut surfaces = Vec::new();
        collect_surfaces(&self.root, &mut surfaces);
        surfaces
    }
}

fn split_node(
    node: &mut PaneNode,
    target_surface: &str,
    direction: SplitDirection,
    new_surface: String,
) -> bool {
    match node {
        PaneNode::Leaf(surface_id) if surface_id == target_surface => {
            let existing = std::mem::take(surface_id);
            let existing = Box::new(PaneNode::Leaf(existing));
            let created = Box::new(PaneNode::Leaf(new_surface));
            let orientation = direction.orientation();
            let (first, second) =
                if direction.insert_first() { (created, existing) } else { (existing, created) };

            *node = PaneNode::Split { orientation, first, second };
            true
        },
        PaneNode::Leaf(_) => false,
        PaneNode::Split { first, second, .. } => {
            split_node(first, target_surface, direction, new_surface.clone())
                || split_node(second, target_surface, direction, new_surface)
        },
    }
}

fn collect_surfaces<'a>(node: &'a PaneNode, surfaces: &mut Vec<&'a str>) {
    match node {
        PaneNode::Leaf(surface_id) => surfaces.push(surface_id),
        PaneNode::Split { first, second, .. } => {
            collect_surfaces(first, surfaces);
            collect_surfaces(second, surfaces);
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{PaneNode, PaneTree, SplitDirection, SplitOrientation};

    #[test]
    fn left_split_is_horizontal_and_inserted_first() {
        let mut tree = PaneTree::new(String::from("surface:1"));
        assert!(tree.split("surface:1", SplitDirection::Left, String::from("surface:2")));

        match tree.root {
            PaneNode::Split { orientation, first, second } => {
                assert_eq!(orientation, SplitOrientation::Horizontal);
                assert_eq!(first, Box::new(PaneNode::Leaf(String::from("surface:2"))));
                assert_eq!(second, Box::new(PaneNode::Leaf(String::from("surface:1"))));
            },
            _ => panic!("expected split node"),
        }
    }

    #[test]
    fn down_split_is_vertical_and_inserted_second() {
        let mut tree = PaneTree::new(String::from("surface:1"));
        assert!(tree.split("surface:1", SplitDirection::Down, String::from("surface:2")));

        match tree.root {
            PaneNode::Split { orientation, first, second } => {
                assert_eq!(orientation, SplitOrientation::Vertical);
                assert_eq!(first, Box::new(PaneNode::Leaf(String::from("surface:1"))));
                assert_eq!(second, Box::new(PaneNode::Leaf(String::from("surface:2"))));
            },
            _ => panic!("expected split node"),
        }
    }
}
