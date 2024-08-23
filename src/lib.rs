//! Vec-backed ID-tree.
//!
//! # Behavior
//!
//! - Trees have at least a root node;
//! - Nodes have zero or more ordered children;
//! - Nodes have at most one parent;
//! - Nodes can be detached (orphaned) but not removed;
//! - Node parent, next sibling, previous sibling, first child and last child
//!   can be accessed in constant time;
//! - All methods perform in constant time;
//! - All iterators perform in linear time.
//!
//! # Examples
//!
//! ```
//! let mut tree = ego_tree::Tree::new('a');
//! let mut root = tree.root_mut();
//! root.append('b');
//! let mut c = root.append('c');
//! c.append('d');
//! c.append('e');
//! ```
//!
//! ```
//! #[macro_use] extern crate ego_tree;
//! # fn main() {
//! let tree = tree!('a' => { 'b', 'c' => { 'd', 'e' } });
//! # }
//! ```

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
)]

use std::fmt::{self, Debug, Formatter};

/// Vec-backed ID-tree.
///
/// Always contains at least a root node.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Tree<T> {
    vec: Vec<Node<T>>,
}

/// Node ID.
///
/// Index into a `Tree`-internal `Vec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node<T> {
    parent: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    children: Option<(NodeId, NodeId)>,
    value: T,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node {
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            children: None,
            value,
        }
    }
}

/// Node reference.
#[derive(Debug)]
pub struct NodeRef<'a, T: 'a> {
    /// Node ID.
    pub id: NodeId,

    /// Tree containing the node.
    pub tree: &'a Tree<T>,

    node: &'a Node<T>,
}

/// Node mutator.
#[derive(Debug)]
pub struct NodeMut<'a, T: 'a> {
    /// Node ID.
    pub id: NodeId,

    /// Tree containing the node.
    pub tree: &'a mut Tree<T>,
}

// Trait implementations regardless of T.

impl<'a, T: 'a> Copy for NodeRef<'a, T> { }
impl<'a, T: 'a> Clone for NodeRef<'a, T> {
    fn clone(&self) -> Self { *self }
}

impl<'a, T: 'a> Eq for NodeRef<'a, T> { }
impl<'a, T: 'a> PartialEq for NodeRef<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.tree as *const _ == other.tree as *const _
            && self.node as *const _ == other.node as *const _
    }
}

impl<T> Tree<T> {
    /// Creates a tree with a root node.
    pub fn new(root: T) -> Self {
        Tree { vec: vec![Node::new(root)] }
    }

    /// Creates a tree with a root node and the specified capacity.
    pub fn with_capacity(root: T, capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(capacity);
        vec.push(Node::new(root));
        Tree { vec }
    }

    /// Returns a reference to the specified node.
    pub fn get(&self, id: NodeId) -> Option<NodeRef<T>> {
        self.vec.get(id.0).map(|node| NodeRef { id, node, tree: self })
    }

    /// Returns a mutator of the specified node.
    pub fn get_mut(&mut self, id: NodeId) -> Option<NodeMut<T>> {
        let exists = self.vec.get(id.0).map(|_| ());
        exists.map(move |_| NodeMut { id, tree: self })
    }

    unsafe fn node(&self, id: NodeId) -> &Node<T> {
        self.vec.get_unchecked(id.0)
    }

    unsafe fn node_mut(&mut self, id: NodeId) -> &mut Node<T> {
        self.vec.get_unchecked_mut(id.0)
    }

    /// Returns a reference to the specified node.
    pub unsafe fn get_unchecked(&self, id: NodeId) -> NodeRef<T> {
        NodeRef { id, node: self.node(id), tree: self }
    }

    /// Returns a mutator of the specified node.
    pub unsafe fn get_unchecked_mut(&mut self, id: NodeId) -> NodeMut<T> {
        NodeMut { id, tree: self }
    }

    /// Returns a reference to the root node.
    pub fn root(&self) -> NodeRef<T> {
        unsafe { self.get_unchecked(NodeId(0)) }
    }

    /// Returns a mutator of the root node.
    pub fn root_mut(&mut self) -> NodeMut<T> {
        unsafe { self.get_unchecked_mut(NodeId(0)) }
    }

    /// Creates an orphan node.
    pub fn orphan(&mut self, value: T) -> NodeMut<T> {
        let id = NodeId(self.vec.len());
        self.vec.push(Node::new(value));
        unsafe { self.get_unchecked_mut(id) }
    }
}

impl<'a, T: 'a> NodeRef<'a, T> {
    /// Returns the value of this node.
    pub fn value(&self) -> &'a T {
        &self.node.value
    }

    /// Returns the parent of this node.
    pub fn parent(&self) -> Option<Self> {
        self.node.parent.map(|id| unsafe { self.tree.get_unchecked(id) })
    }

    /// Returns the previous sibling of this node.
    pub fn prev_sibling(&self) -> Option<Self> {
        self.node.prev_sibling.map(|id| unsafe { self.tree.get_unchecked(id) })
    }

    /// Returns the next sibling of this node.
    pub fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling.map(|id| unsafe { self.tree.get_unchecked(id) })
    }

    /// Returns the first child of this node.
    pub fn first_child(&self) -> Option<Self> {
        self.node.children.map(|(id, _)| unsafe { self.tree.get_unchecked(id) })
    }

    /// Returns the last child of this node.
    pub fn last_child(&self) -> Option<Self> {
        self.node.children.map(|(_, id)| unsafe { self.tree.get_unchecked(id) })
    }

    /// Returns the index of the given child or None if child doesn't exist.
    pub fn index_of_child(&self, child: &NodeRef<T>) -> Option<usize> {
        for (i, ref each) in self.children().enumerate() {
            if each == child {
                return Some(i)
            }
        }
        None
    }

    /// Returns true if this node has siblings.
    pub fn has_siblings(&self) -> bool {
        self.node.prev_sibling.is_some() || self.node.next_sibling.is_some()
    }

    /// Returns true if this node has children.
    pub fn has_children(&self) -> bool {
        self.node.children.is_some()
    }
}

impl<'a, T: 'a> NodeMut<'a, T> {
    fn node(&mut self) -> &mut Node<T> {
        unsafe { self.tree.node_mut(self.id) }
    }

    /// Returns the value of this node.
    pub fn value(&mut self) -> &mut T {
        &mut self.node().value
    }

    /// Returns the parent of this node.
    pub fn parent(&mut self) -> Option<NodeMut<T>> {
        let id = self.node().parent;
        id.map(move |id| unsafe { self.tree.get_unchecked_mut(id) })
    }

    /// Returns the previous sibling of this node.
    pub fn prev_sibling(&mut self) -> Option<NodeMut<T>> {
        let id = self.node().prev_sibling;
        id.map(move |id| unsafe { self.tree.get_unchecked_mut(id) })
    }

    /// Returns the next sibling of this node.
    pub fn next_sibling(&mut self) -> Option<NodeMut<T>> {
        let id = self.node().next_sibling;
        id.map(move |id| unsafe { self.tree.get_unchecked_mut(id) })
    }

    /// Returns the first child of this node.
    pub fn first_child(&mut self) -> Option<NodeMut<T>> {
        let ids = self.node().children;
        ids.map(move |(id, _)| unsafe { self.tree.get_unchecked_mut(id) })
    }

    /// Returns the last child of this node.
    pub fn last_child(&mut self) -> Option<NodeMut<T>> {
        let ids = self.node().children;
        ids.map(move |(_, id)| unsafe { self.tree.get_unchecked_mut(id) })
    }

    /// Returns true if this node has siblings.
    pub fn has_siblings(&self) -> bool {
        unsafe { self.tree.get_unchecked(self.id).has_siblings() }
    }

    /// Returns true if this node has children.
    pub fn has_children(&self) -> bool {
        unsafe { self.tree.get_unchecked(self.id).has_children() }
    }

    /// Appends a new child to this node.
    pub fn append(&mut self, value: T) -> NodeMut<T> {
        let id = self.tree.orphan(value).id;
        self.append_id(id)
    }

    /// Prepends a new child to this node.
    pub fn prepend(&mut self, value: T) -> NodeMut<T> {
        let id = self.tree.orphan(value).id;
        self.prepend_id(id)
    }
    
    /// Insert a new child into this node at given index.
    /// This function may take up to linear time in worst case scenarios.
    ///
    /// # Panics
    ///
    /// Panics if `index` is not valid.
    pub fn insert(&mut self, value: T, index: usize) -> NodeMut<T> {
        let id = self.tree.orphan(value).id;
        self.insert_id(id, index)
    }

    /// Inserts a new sibling before this node.
    ///
    /// # Panics
    ///
    /// Panics if this node is an orphan.
    pub fn insert_before(&mut self, value: T) -> NodeMut<T> {
        let id = self.tree.orphan(value).id;
        self.insert_id_before(id)
    }

    /// Inserts a new sibling after this node.
    ///
    /// # Panics
    ///
    /// Panics if this node is an orphan.
    pub fn insert_after(&mut self, value: T) -> NodeMut<T> {
        let id = self.tree.orphan(value).id;
        self.insert_id_after(id)
    }

    /// Detaches this node from its parent.
    pub fn detach(&mut self) {
        let parent_id = match self.node().parent {
            Some(id) => id,
            None => return,
        };
        let prev_sibling_id = self.node().prev_sibling;
        let next_sibling_id = self.node().next_sibling;

        {
            self.node().parent = None;
            self.node().prev_sibling = None;
            self.node().next_sibling = None;
        }

        if let Some(id) = prev_sibling_id {
            unsafe { self.tree.node_mut(id).next_sibling = next_sibling_id; }
        }
        if let Some(id) = next_sibling_id {
            unsafe { self.tree.node_mut(id).prev_sibling = prev_sibling_id; }
        }

        let parent = unsafe { self.tree.node_mut(parent_id) };
        let (first_child_id, last_child_id) = parent.children.unwrap();
        if first_child_id == last_child_id {
            parent.children = None;
        } else if first_child_id == self.id {
            parent.children = Some((next_sibling_id.unwrap(), last_child_id));
        } else if last_child_id == self.id {
            parent.children = Some((first_child_id, prev_sibling_id.unwrap()));
        }
    }

    /// Appends a child to this node.
    ///
    /// # Panics
    ///
    /// Panics if `new_child_id` is not valid.
    pub fn append_id(&mut self, new_child_id: NodeId) -> NodeMut<T> {
        let last_child_id = self.node().children.map(|(_, id)| id);
        {
            let mut new_child = self.tree.get_mut(new_child_id).unwrap();
            new_child.detach();
            new_child.node().parent = Some(self.id);
            new_child.node().prev_sibling = last_child_id;
        }

        if let Some(id) = last_child_id {
            unsafe { self.tree.node_mut(id).next_sibling = Some(new_child_id); }
        }

        {
            if let Some((first_child_id, _)) = self.node().children {
                self.node().children = Some((first_child_id, new_child_id));
            } else {
                self.node().children = Some((new_child_id, new_child_id));
            }
        }

        unsafe { self.tree.get_unchecked_mut(new_child_id) }
    }

    /// Insert a child into this node at given index.
    /// This function may take up to linear time in worst case scenarios.
    ///
    /// # Panics
    ///
    /// Panics if `new_child_id` or `index` are not valid.
    pub fn insert_id(&mut self, new_child_id: NodeId, index: usize) -> NodeMut<T> {
        if index == 0 {
            return self.prepend_id(new_child_id)
        }

        let mut pre_sibling: NodeMut<T> = unsafe {
            self.tree
            .get_unchecked(self.id)
            .children()
            .nth(index - 1) // worst case O(n)
            .map(|node| node.id)
            .map(|id| self.tree.get_unchecked_mut(id))
            .expect(format!("No child found at index {}", index-1).as_str())
        };

        pre_sibling.insert_id_after(new_child_id);
        unsafe { self.tree.get_unchecked_mut(new_child_id) }
    }

    /// Prepends a child to this node.
    ///
    /// # Panics
    ///
    /// Panics if `new_child_id` is not valid.
    pub fn prepend_id(&mut self, new_child_id: NodeId) -> NodeMut<T> {
        let first_child_id = self.node().children.map(|(id, _)| id);
        {
            let mut new_child = self.tree.get_mut(new_child_id).unwrap();
            new_child.detach();
            new_child.node().parent = Some(self.id);
            new_child.node().next_sibling = first_child_id;
        }

        if let Some(id) = first_child_id {
            unsafe { self.tree.node_mut(id).prev_sibling = Some(new_child_id); }
        }

        {
            if let Some((_, last_child_id)) = self.node().children {
                self.node().children = Some((new_child_id, last_child_id));
            } else {
                self.node().children = Some((new_child_id, new_child_id));
            }
        }

        unsafe { self.tree.get_unchecked_mut(new_child_id) }
    }

    /// Inserts a sibling before this node.
    ///
    /// # Panics
    ///
    /// - Panics if `new_sibling_id` is not valid.
    /// - Panics if this node is an orphan.
    pub fn insert_id_before(&mut self, new_sibling_id: NodeId) -> NodeMut<T> {
        let parent_id = self.node().parent.unwrap();
        let prev_sibling_id = self.node().prev_sibling;

        {
            let mut new_sibling = self.tree.get_mut(new_sibling_id).unwrap();
            new_sibling.node().parent = Some(parent_id);
            new_sibling.node().prev_sibling = prev_sibling_id;
            new_sibling.node().next_sibling = Some(self.id);
        }

        if let Some(id) = prev_sibling_id {
            unsafe { self.tree.node_mut(id).next_sibling = Some(new_sibling_id); }
        }

        self.node().prev_sibling = Some(new_sibling_id);

        {
            let parent = unsafe { self.tree.node_mut(parent_id) };
            let (first_child_id, last_child_id) = parent.children.unwrap();
            if first_child_id == self.id {
                parent.children = Some((new_sibling_id, last_child_id));
            }
        }

        unsafe { self.tree.get_unchecked_mut(new_sibling_id) }
    }

    /// Inserts a sibling after this node.
    ///
    /// # Panics
    ///
    /// - Panics if `new_sibling_id` is not valid.
    /// - Panics if this node is an orphan.
    pub fn insert_id_after(&mut self, new_sibling_id: NodeId) -> NodeMut<T> {
        let parent_id = self.node().parent.unwrap();
        let next_sibling_id = self.node().next_sibling;

        {
            let mut new_sibling = self.tree.get_mut(new_sibling_id).unwrap();
            new_sibling.node().parent = Some(parent_id);
            new_sibling.node().prev_sibling = Some(self.id);
            new_sibling.node().next_sibling = next_sibling_id;
        }

        if let Some(id) = next_sibling_id {
            unsafe { self.tree.node_mut(id).prev_sibling = Some(new_sibling_id); }
        }

        self.node().next_sibling = Some(new_sibling_id);

        {
            let parent = unsafe { self.tree.node_mut(parent_id) };
            let (first_child_id, last_child_id) = parent.children.unwrap();
            if last_child_id == self.id {
                parent.children = Some((first_child_id, new_sibling_id));
            }
        }

        unsafe { self.tree.get_unchecked_mut(new_sibling_id) }
    }

    /// Reparents the children of a node, appending them to this node.
    ///
    /// # Panics
    ///
    /// Panics if `from_id` is not valid.
    pub fn reparent_from_id_append(&mut self, from_id: NodeId) {
        let new_child_ids = {
            let mut from = self.tree.get_mut(from_id).unwrap();
            match from.node().children.take() {
                Some(ids) => ids,
                None => return,
            }
        };

        unsafe {
            self.tree.node_mut(new_child_ids.0).parent = Some(self.id);
            self.tree.node_mut(new_child_ids.1).parent = Some(self.id);
        }

        if self.node().children.is_none() {
            self.node().children = Some(new_child_ids);
            return;
        }

        let old_child_ids = self.node().children.unwrap();
        unsafe {
            self.tree.node_mut(old_child_ids.1).next_sibling = Some(new_child_ids.0);
            self.tree.node_mut(new_child_ids.0).prev_sibling = Some(old_child_ids.1);
        }

        self.node().children = Some((old_child_ids.0, new_child_ids.1));
    }

    /// Reparents the children of a node, prepending them to this node.
    ///
    /// # Panics
    ///
    /// Panics if `from_id` is not valid.
    pub fn reparent_from_id_prepend(&mut self, from_id: NodeId) {
        let new_child_ids = {
            let mut from = self.tree.get_mut(from_id).unwrap();
            match from.node().children.take() {
                Some(ids) => ids,
                None => return,
            }
        };

        unsafe {
            self.tree.node_mut(new_child_ids.0).parent = Some(self.id);
            self.tree.node_mut(new_child_ids.1).parent = Some(self.id);
        }

        if self.node().children.is_none() {
            self.node().children = Some(new_child_ids);
            return;
        }

        let old_child_ids = self.node().children.unwrap();
        unsafe {
            self.tree.node_mut(old_child_ids.0).prev_sibling = Some(new_child_ids.1);
            self.tree.node_mut(new_child_ids.1).next_sibling = Some(old_child_ids.0);
        }

        self.node().children = Some((new_child_ids.0, old_child_ids.1));
    }
}

impl<'a, T: 'a> From<NodeMut<'a, T>> for NodeRef<'a, T> {
    fn from(node: NodeMut<'a, T>) -> Self {
        unsafe { node.tree.get_unchecked(node.id) }
    }
}

/// Iterators.
pub mod iter;

/// Creates a tree from expressions.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate ego_tree;
/// # fn main() {
/// let tree = tree!("root");
/// # }
/// ```
///
/// ```
/// #[macro_use] extern crate ego_tree;
/// # fn main() {
/// let tree = tree! {
///     "root" => {
///         "child a",
///         "child b" => {
///             "grandchild a",
///             "grandchild b",
///         },
///         "child c",
///     }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! tree {
    (@ $n:ident { }) => { };

    // Last leaf.
    (@ $n:ident { $value:expr }) => {
        { $n.append($value); }
    };

    // Leaf.
    (@ $n:ident { $value:expr, $($tail:tt)* }) => {
        {
            $n.append($value);
            tree!(@ $n { $($tail)* });
        }
    };

    // Last node with children.
    (@ $n:ident { $value:expr => $children:tt }) => {
        {
            let mut node = $n.append($value);
            tree!(@ node $children);
        }
    };

    // Node with children.
    (@ $n:ident { $value:expr => $children:tt, $($tail:tt)* }) => {
        {
            {
                let mut node = $n.append($value);
                tree!(@ node $children);
            }
            tree!(@ $n { $($tail)* });
        }
    };

    ($root:expr) => { $crate::Tree::new($root) };

    ($root:expr => $children:tt) => {
        {
            let mut tree = $crate::Tree::new($root);
            {
                let mut node = tree.root_mut();
                tree!(@ node $children);
            }
            tree
        }
    };
}

impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use iter::Edge;
        if f.alternate() {
            write!(f, "Tree {{")?;
            for edge in self.root().traverse() {
                match edge {
                    Edge::Open(node) if node.has_children() => {
                        write!(f, " {:?} => {{", node.value())?;
                    },
                    Edge::Open(node) if node.next_sibling().is_some() => {
                        write!(f, " {:?},", node.value())?;
                    },
                    Edge::Open(node) => {
                        write!(f, " {:?}", node.value())?;
                    },
                    Edge::Close(node) if node.has_children() => {
                        if node.next_sibling().is_some() {
                            write!(f, " }},")?;
                        } else {
                            write!(f, " }}")?;
                        }
                    },
                    _ => {},
                }
            }
            write!(f, " }}")
        } else {
            f.debug_struct("Tree").field("vec", &self.vec).finish()
        }
    }
}
