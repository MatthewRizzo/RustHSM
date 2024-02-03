//! File implementing a generic tree that is depth, first searchable
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::errors::{HSMError, HSMResult};

/// Trait that needs to be implemented by NODE consumers of this tree.
/// Abstracts the operation of comparing 2 nodes in a tree.
/// So long as your node implements it, and that node is used for your struct
/// containing the tree in this file, it should all work!
pub trait NodeDataConstraints {}

/// Wrapper around the reference tree node's hold to their data
pub type TreeNodeDataRef<T> = Rc<RefCell<T>>;
/// Wrapper around references to tree nodes with a generic data type
pub(crate) type TreeNodeRef<DataType> = Rc<RefCell<TreeNode<DataType>>>;
type GenericTreeNodeRef<Node> = Rc<RefCell<Node>>;

/// Generic tree that can be used anywhere! As long as the traits of its nodes
/// are met.
/// Most likely requires an LTM to hold all the node's as the tree only accepts
/// references.
/// The tree will own your nodes for you, but you can keep owning the data they
/// hold!
pub struct Tree<Node> {
    // todo try to convert this to dynamic dispatch
    nodes: Vec<GenericTreeNodeRef<Node>>,
    num_nodes: u16,
}

/// The nodes of the tree
/// Nodes are wholly owned by the tree.
/// Nodes ONLY have references to their data.
/// 'a = lifetime of the data within a node
/// todo - rewrite using Rc<RefCell<T>> from
/// https://rusty-ferris.pages.dev/blog/binary-tree-sum-of-values/
pub struct TreeNode<NodeDataType: NodeDataConstraints + PartialEq> {
    data: TreeNodeDataRef<NodeDataType>,

    /// The root node of the entire tree. None if this node IS the root.
    root_node: Option<TreeNodeRef<NodeDataType>>,
    /// The parent node of this node. None if this node IS the root.
    parent: Option<TreeNodeRef<NodeDataType>>,
}

impl<'a, Node> Tree<Node>
where
    Node: NodeOperations<NodeImpl = Node> + PartialEq + 'a,
{
    /// Use this to create tree
    /// The id of the root node will always be 0
    pub fn create_tree(root_node_data: TreeNodeDataRef<Node::NodeDataType>) -> Self {
        let root_node = Node::new(root_node_data, None, None);
        let ref_root_node = Rc::new(RefCell::new(root_node));
        Tree {
            nodes: vec![ref_root_node],
            num_nodes: 1,
        }
    }

    /// Adds the node and returns its node id
    // pub fn add_node(
    //     &mut self,
    //     node_data: TreeNodeDataRef<Node::NodeDataType>,
    //     parent_node_id: u16,
    // ) -> u16 {
    //     let node_id = self.num_nodes;
    //     self.num_nodes += 1;
    //     let node = self.create_node(node_data, parent_node_id);
    //     self.nodes.push(node);
    //     node_id
    // }

    fn get_node_id_from_node(
        &self,
        other_node_ref: TreeNodeDataRef<Node::NodeDataType>,
    ) -> Option<u16> {
        for (idx, node_ref) in self.nodes.iter().enumerate() {
            let node = node_ref.borrow();
            if node.is_data_contained_the_same(other_node_ref.clone()) {
                return Some(idx as u16);
            }
        }

        None
    }

    pub fn add_node_with_parent_data(
        &mut self,
        node_data: TreeNodeDataRef<Node::NodeDataType>,
        parent_node: TreeNodeDataRef<Node::NodeDataType>,
    ) -> HSMResult<u16> {
        let node_id = self.num_nodes;
        self.num_nodes += 1;

        let parent_node_id: u16 = self
            .get_node_id_from_node(parent_node.clone())
            .ok_or_else(|| HSMError::GenericError("Could not find node from data!".to_string()))?;

        let node = self.create_node(node_data.clone(), parent_node_id);
        self.nodes.push(node);
        Ok(node_id)
    }

    pub fn add_node_with_parent_node(
        &mut self,
        node_data: TreeNodeDataRef<Node>,
        parent_node: TreeNodeDataRef<Node>,
    ) -> HSMResult<u16> {
        let node_id = self.num_nodes;
        self.num_nodes += 1;

        let parent_node_id: u16 = self
            .get_node_id_from_node(parent_node.borrow().get_node_data())
            .ok_or_else(|| HSMError::GenericError("Could not find node from data!".to_string()))?;

        let node = self.create_node(node_data.borrow().get_node_data(), parent_node_id);
        self.nodes.push(node);
        Ok(node_id)
    }

    /// Node's do NOT own their data!
    fn create_node(
        &'a self,
        data: TreeNodeDataRef<Node::NodeDataType>,
        parent_node_id: u16,
    ) -> GenericTreeNodeRef<Node> {
        let parent_node = self.get_node_by_id(parent_node_id);
        let node = Node::new(data, parent_node, Some(self.get_root_node()));
        Rc::new(RefCell::new(node))
    }

    /// Return the node based on that data it holds as a key
    /// TODO - remove if unused
    pub fn find_node_by_data(
        &'a self,
        node_data: TreeNodeDataRef<Node::NodeDataType>,
    ) -> Option<&GenericTreeNodeRef<Node>> {
        for node in &self.nodes {
            if node.borrow().is_data_contained_the_same(node_data.clone()) {
                return Some(node);
            }
        }
        return None;
    }

    pub fn get_root_node(&'a self) -> GenericTreeNodeRef<Node> {
        // Rc::new(RefCell::new( self.nodes[0] ) )
        self.nodes[0].clone()
    }

    /// Inspiration: https://stackoverflow.com/a/61512383/14810215
    /// Finds the path between 2 nodes. Includes the ending node, but not the starting node!
    fn find_path_between_nodes(
        &'a self,
        start_node_id: u16,
        end_node_id: u16,
    ) -> Vec<Rc<RefCell<Node>>> {
        // todo - confirm there is no way it is not None
        let start_node = self.get_node_by_id(start_node_id).unwrap();
        let end_node = self.get_node_by_id(end_node_id).unwrap();

        let mut start_path_to_root = start_node.borrow().get_path_to_root();
        let mut destination_path_to_root = end_node.borrow().get_path_to_root();

        // the last node in common between the paths
        let mut last_common_node = None;

        // Compare the two paths, starting from the ends of the paths (where the root is)
        // as long as they are the same, remove that common node from both paths.
        while start_path_to_root.len() > 0 && destination_path_to_root.len() > 0 {
            let starting_path_node = start_path_to_root[start_path_to_root.len() - 1].clone();
            let ending_path_node =
                destination_path_to_root[destination_path_to_root.len() - 1].clone();
            if starting_path_node == ending_path_node {
                last_common_node = start_path_to_root.pop();
                destination_path_to_root.pop();
            } else {
                // stop once the path's diverge
                break;
            }
        }

        // Then Reverse the second path, and join the two paths
        // putting the last removed node in between the two.
        let mut common_to_dest = destination_path_to_root.clone();
        common_to_dest.reverse();

        let mut full_path = vec![start_node];
        full_path.append(&mut start_path_to_root);

        // add the last link in chain between the nodes to path
        if last_common_node.is_some() {
            full_path.push(last_common_node.unwrap());
        }
        full_path.append(&mut common_to_dest);
        full_path.push(end_node);

        return full_path;
    }
}

/// Operations a tree MUST implement to be valid. Used to break the circular
/// dependency between node's and tree's trait(s)
pub trait TreeOperations {
    type NodeImpl: NodeOperations;
    type NodeDataType: NodeDataConstraints;

    fn get_node_by_id(&self, id: u16) -> Option<GenericTreeNodeRef<Self::NodeImpl>>;
}

impl<Node> TreeOperations for Tree<Node>
where
    Node: NodeOperations,
{
    type NodeImpl = Node;
    type NodeDataType = Node::NodeDataType;

    fn get_node_by_id(&self, id: u16) -> Option<GenericTreeNodeRef<Self::NodeImpl>> {
        if id < self.num_nodes {
            let node = self
                .nodes
                .get(id as usize)
                .expect(format!("Provided id {} for a node that does not exist!", id).as_str())
                .clone();

            Some(node)
        } else {
            None
        }
    }
}

/// Private trait operations that are needed for this to work, but that outside
/// consumers should NEVER know exist
pub trait NodeOperations {
    type NodeImpl;
    type NodeDataType: NodeDataConstraints;

    // Get the path to root, including root
    fn get_path_to_root(&self) -> Vec<Rc<RefCell<Self::NodeImpl>>>;

    /// Return true if the data contained in this node matches the data provided
    fn is_data_contained_the_same(&self, data_key: Rc<RefCell<Self::NodeDataType>>) -> bool;

    // Private abstract method for creating a node.
    // Used by the tree to help add to itself
    fn new(
        data: TreeNodeDataRef<Self::NodeDataType>,
        parent_node: Option<Rc<RefCell<Self::NodeImpl>>>,
        root_node: Option<Rc<RefCell<Self::NodeImpl>>>,
    ) -> Self;

    fn get_node_data(&self) -> Rc<RefCell<Self::NodeDataType>>;
    /// Returns the parent node if it exists
    /// If we are root, return None
    fn get_node_parent(&self) -> Option<Rc<RefCell<Self::NodeImpl>>>;
}

impl<'a, NodeDataType> PartialEq for TreeNode<NodeDataType>
where
    NodeDataType: NodeDataConstraints,
    NodeDataType: PartialEq,
{
    fn eq(&self, other: &TreeNode<NodeDataType>) -> bool {
        self.data == other.data
    }
}

impl<NodeDataType> NodeOperations for TreeNode<NodeDataType>
where
    NodeDataType: NodeDataConstraints + PartialEq,
{
    type NodeImpl = TreeNode<NodeDataType>;
    type NodeDataType = NodeDataType;

    /// Returns the path to the root node.
    /// Last element should be root.
    /// First element is NOT self / starting node
    fn get_path_to_root<'a>(&self) -> Vec<TreeNodeRef<NodeDataType>> {
        let mut visited: Vec<Rc<RefCell<TreeNode<NodeDataType>>>> = vec![];

        if self.parent.is_some() {
            Self::get_path_to_root_inner(&mut visited, self.parent.clone().unwrap());
        }

        visited
    }

    fn is_data_contained_the_same(&self, data_key: TreeNodeDataRef<Self::NodeDataType>) -> bool {
        return *self.data.borrow() == *data_key.borrow();
    }

    fn new(
        data: TreeNodeDataRef<NodeDataType>,
        parent_node: Option<TreeNodeRef<NodeDataType>>,
        root_node: Option<TreeNodeRef<NodeDataType>>,
    ) -> Self {
        TreeNode {
            data,
            parent: parent_node,
            root_node: root_node,
        }
    }

    fn get_node_data(&self) -> Rc<RefCell<Self::NodeDataType>> {
        self.data.clone()
    }

    fn get_node_parent(&self) -> Option<Rc<RefCell<Self::NodeImpl>>> {
        self.parent.clone()
    }
}

impl<NodeDataType> TreeNode<NodeDataType>
where
    NodeDataType: NodeDataConstraints,
    NodeDataType: PartialEq,
{
    /// Gets the path to root, adding each node along the way to visited!
    fn get_path_to_root_inner(
        visited: &mut Vec<Rc<RefCell<TreeNode<NodeDataType>>>>,
        current_node: Rc<RefCell<TreeNode<NodeDataType>>>,
    ) {
        visited.push(current_node.clone());
        if current_node.borrow().parent.is_some() {
            let next_node: Rc<RefCell<TreeNode<NodeDataType>>> =
                current_node.borrow().parent.as_ref().unwrap().to_owned();
            Self::get_path_to_root_inner(visited, next_node)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::clone;

    use super::*;

    #[derive(PartialEq, Debug, Clone)]
    struct TestData {
        fake_data: u8,
    }

    impl NodeDataConstraints for TestData {}

    struct TestNodes {
        root_node: TreeNode<TestData>,
        node1: TreeNode<TestData>,
        node2: TreeNode<TestData>,
        node3: TreeNode<TestData>,
    }

    #[test]
    fn test_create_tree() {
        let root_data = Rc::new(RefCell::new(TestData { fake_data: 1 }));
        let node1 = Rc::new(RefCell::new(TestData { fake_data: 2 }));
        let node2 = Rc::new(RefCell::new(TestData { fake_data: 3 }));
        let node3 = Rc::new(RefCell::new(TestData { fake_data: 4 }));

        let mut tree: Tree<TreeNode<TestData>> = Tree::create_tree(root_data.clone());
        let root_node = tree.get_root_node();

        assert_eq!(tree.num_nodes, 1);
        assert_eq!(tree.get_node_by_id(0).unwrap().borrow().data, root_data);

        let node1_id = tree.add_node_with_parent_data(node1, root_data).expect("");
        let node2_id = tree.add_node_with_parent_data(node2, root_data).expect("");
        let child_node3_id = tree
            .add_node_with_parent_data(node3.clone(), node1)
            .expect("");

        assert_eq!(node1_id, 1);
        assert_eq!(node2_id, 2);
        assert_eq!(child_node3_id, 3);

        assert_eq!(tree.get_node_by_id(node1_id).unwrap().borrow().data, node1);
        assert_eq!(tree.get_node_by_id(node2_id).unwrap().borrow().data, node2);
        assert_eq!(
            tree.get_node_by_id(child_node3_id).unwrap().borrow().data,
            node3
        );

        // test pathing between nodes

        let node_1_to_2_path = tree.find_path_between_nodes(1, 2);
        assert_eq!(node_1_to_2_path.len(), 3, "Nodes in path from 1->2 = 3");
        assert_eq!(
            node_1_to_2_path[0].borrow().data,
            node1,
            "Expected node 1 data"
        );
        assert_eq!(
            node_1_to_2_path[1].borrow().data,
            root_data,
            "Expected root node data"
        );
    }

    #[test]
    fn test_get_path_to_root() {
        let root_node = Rc::new(RefCell::new(TestData { fake_data: 1 }));
        let node1 = Rc::new(RefCell::new(TestData { fake_data: 2 }));
        let data2 = Rc::new(RefCell::new(TestData { fake_data: 3 }));
        let node3_node1_child = Rc::new(RefCell::new(TestData { fake_data: 4 }));

        let mut tree: Tree<TreeNode<TestData>> = Tree::create_tree(root_node.clone());

        let node1_id = tree
            .add_node_with_parent_data(node1.clone(), root_node)
            .expect("");
        let node2_id = tree
            .add_node_with_parent_data(data2.clone(), root_node)
            .expect("");
        let node3_id = tree
            .add_node_with_parent_data(node3_node1_child.clone(), node1.clone())
            .expect("");

        let node1_to_root = tree
            .get_node_by_id(node1_id)
            .unwrap()
            .borrow()
            .get_path_to_root();
        let node2_to_root = tree
            .get_node_by_id(node2_id)
            .unwrap()
            .borrow()
            .get_path_to_root();
        let node3_to_root = tree
            .get_node_by_id(node3_id)
            .unwrap()
            .borrow()
            .get_path_to_root();

        assert_eq!(node1_to_root.len(), 1);
        assert_eq!(node2_to_root.len(), 1);
        assert_eq!(node3_to_root.len(), 2);

        assert_eq!(node1_to_root[0].borrow().data, root_node);
        assert_eq!(node2_to_root[0].borrow().data, root_node);

        assert_eq!(node3_to_root[0].borrow().data, node1);
        assert_eq!(node3_to_root[1].borrow().data, root_node);
    }

    // todo - more tests
}
