//! File implementing a generic tree that is depth, first searchable

// todo - consider moving this tree impl to its own library

/// Trait that needs to be implemented by NODE consumers of this tree.
/// Abstracts the operation of comparing 2 nodes in a tree.
/// So long as your node implements it, and that node is used for your struct
/// containing the tree in this file, it should all work!
pub trait NodeDataConstraints {}

/// Generic tree that can be used anywhere! As long as the traits of its nodes
/// are met.
/// Most likely requires an LTM to hold all the node's as the tree only accepts
/// references.
pub(crate) struct Tree<'a, Node>
where
    Node: NodeOperations<'a>,
{
    // maybe nodes should be owned references??
    nodes: Vec<&'a Node>,
    num_nodes: u32,
}

// The nodes of the tree
// todo - consider making private
pub struct TreeNode<'a, NodeDataType: NodeDataConstraints + PartialEq> {
    data: NodeDataType,

    /// The root node of the entire tree. None if this node IS the root.
    root_node: Option<&'a TreeNode<'a, NodeDataType>>,
    /// The parent node of this node. None if this node IS the root.
    parent: Option<&'a TreeNode<'a, NodeDataType>>,
}

impl<'a, Node> Tree<'a, Node>
where
    Node: NodeOperations<'a, NodeImpl = Node> + PartialEq,
{
    /// Use this to create tree
    pub fn create_tree(root_node: &'a Node) -> Tree<'a, Node> {
        Tree {
            nodes: vec![root_node],
            num_nodes: 1,
        }
    }

    /// Adds the node and returns its node id
    pub fn add_node(&mut self, node: &'a Node) -> u32 {
        let node_id = self.num_nodes;
        self.num_nodes += 1;
        self.nodes.push(node);
        node_id
    }

    pub fn create_node(&self, data: Node::NodeDataType, parent_node: &'a Node) -> Node {
        Node::new(data, Some(parent_node), Some(self.get_root_node()))
    }

    /// Inspiration: https://stackoverflow.com/a/61512383/14810215
    /// Finds the path between 2 nodes. Includes the starting and ending node.
    fn find_path_between_nodes(&'a self, start_node_id: u32, end_node_id: u32) -> Vec<&'a Node> {
        let start_node = self.get_node_by_id(start_node_id);
        let end_node = self.get_node_by_id(end_node_id);

        let mut start_path_to_root = start_node.get_path_to_root();
        let mut destination_path_to_root = end_node.get_path_to_root();

        // the last node in common between the paths
        let mut last_common_node = None;

        // Compare the two paths, starting from the ends of the paths (where the root is)
        // as long as they are the same, remove that common node from both paths.
        while start_path_to_root.len() > 0 && destination_path_to_root.len() > 0 {
            let starting_path_node = start_path_to_root[start_path_to_root.len() - 1];
            let ending_path_node = destination_path_to_root[destination_path_to_root.len() - 1];
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

        let mut full_path: Vec<&Node> = start_path_to_root;

        // add the last link in chain between the nodes to path
        if last_common_node.is_some() {
            full_path.push(last_common_node.unwrap());
        }
        full_path.append(&mut common_to_dest);

        return full_path;
    }

    fn get_root_node(&self) -> &'a Node {
        self.nodes[0]
    }
}

/// Operations a tree MUST implement to be valid. Used to break the circular
/// dependency between node's and tree's trait(s)
trait TreeOperations<'a> {
    type NodeImpl: NodeOperations<'a>;
    type NodeDataType: NodeDataConstraints;

    fn get_node_by_id(&self, id: u32) -> &Self::NodeImpl;

    /// Creates the root node for a new tree and returns it
    // fn create_root_node<DataType>(data: Self::Data) -> Self::NodeImpl;
    fn create_root_node(data: Self::NodeDataType) -> Self::NodeImpl;
}

impl<'a, Node> TreeOperations<'a> for Tree<'a, Node>
where
    Node: NodeOperations<'a>,
{
    type NodeImpl = Node;
    type NodeDataType = Node::NodeDataType;

    fn get_node_by_id(&self, id: u32) -> &Self::NodeImpl {
        self.nodes
            .get(id as usize)
            .expect(format!("Provided id {} for a node that does not exist!", id).as_str())
    }

    // fn create_root_node(data: Self::Data) -> Self::NodeImpl {
    fn create_root_node(data: Self::NodeDataType) -> Self::NodeImpl {
        Node::new(data, None, None)
    }
}

/// Private trait operations that are needed for this to work, but that outside
/// consumers should NEVER know exist
pub(crate) trait NodeOperations<'a> {
    type NodeImpl;
    type NodeDataType: NodeDataConstraints;

    // Get the path to root, including root
    fn get_path_to_root(&'a self) -> Vec<&'a Self::NodeImpl>;

    // Private abstract method for creating a node.
    // Used by the tree to help add to itself
    fn new(
        data: Self::NodeDataType,
        parent_node: Option<&'a Self::NodeImpl>,
        root_node: Option<&'a Self::NodeImpl>,
    ) -> Self;
}

impl<'a, NodeDataType> PartialEq for TreeNode<'a, NodeDataType>
where
    NodeDataType: NodeDataConstraints,
    NodeDataType: PartialEq,
{
    fn eq(&self, other: &TreeNode<'a, NodeDataType>) -> bool {
        self.data == other.data
    }
}

impl<'a, NodeDataType> NodeOperations<'a> for TreeNode<'a, NodeDataType>
where
    NodeDataType: NodeDataConstraints,
    NodeDataType: PartialEq,
{
    type NodeImpl = TreeNode<'a, NodeDataType>;
    type NodeDataType = NodeDataType;

    /// Returns the path to the root node.
    /// last element should be root
    /// first element is self
    /// todo - unit test this
    fn get_path_to_root(&'a self) -> Vec<&'a Self::NodeImpl> {
        let mut previous_node: &'a TreeNode<'a, NodeDataType> = self;
        let mut current_node = Some(self);

        let mut visited: Vec<&'a TreeNode<'a, NodeDataType>> = vec![];

        while current_node.is_some() {
            previous_node = current_node.unwrap();
            visited.push(previous_node);
            current_node = current_node.unwrap().parent;
        }

        visited
    }

    fn new(
        data: NodeDataType,
        parent_node: Option<&'a Self::NodeImpl>,
        root_node: Option<&'a Self::NodeImpl>,
    ) -> Self {
        TreeNode {
            data,
            parent: parent_node,
            root_node: root_node,
        }
    }
}

// todo - make unit tests here
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug, Clone)]
    struct TestData {
        fake_data: u8,
    }

    impl NodeDataConstraints for TestData {}

    struct TestNodes<'a> {
        root_node: TreeNode<'a, TestData>,
        node1: TreeNode<'a, TestData>,
        node2: TreeNode<'a, TestData>,
        node3: TreeNode<'a, TestData>,
    }

    #[test]
    fn test_create_tree() {
        let root_data = TestData { fake_data: 1 };
        let data1 = TestData { fake_data: 2 };
        let data2 = TestData { fake_data: 3 };
        let data3 = TestData { fake_data: 4 };

        let root_node: TreeNode<TestData> = Tree::create_root_node(root_data.clone());
        let mut tree: Tree<TreeNode<TestData>> = Tree::create_tree(&root_node);

        assert_eq!(tree.num_nodes, 1);
        assert_eq!(tree.get_node_by_id(0).data, root_data);

        let node1 = tree.create_node(data1.clone(), &root_node);
        let node2 = tree.create_node(data2.clone(), &root_node);
        let child_node3 = tree.create_node(data3.clone(), &node1);

        let node1_id = tree.add_node(&node1);
        let node2_id = tree.add_node(&node2);
        let child_node3_id = tree.add_node(&child_node3);

        assert_eq!(node1_id, 1);
        assert_eq!(node2_id, 2);
        assert_eq!(child_node3_id, 3);

        assert_eq!(tree.get_node_by_id(node1_id).data, data1);
        assert_eq!(tree.get_node_by_id(node2_id).data, data2);
        assert_eq!(tree.get_node_by_id(child_node3_id).data, data3);

        // test pathing between nodes

        let node_1_to_2_path = tree.find_path_between_nodes(1, 2);
        assert_eq!(node_1_to_2_path.len(), 3, "Nodes in path from 1->2 = 3");
        assert_eq!(node_1_to_2_path[0].data, node1.data, "Expected node 1 data");
        assert_eq!(
            node_1_to_2_path[1].data, root_node.data,
            "Expected root node data"
        );
    }

    #[test]
    fn test_get_path_to_root() {
        let root_data = TestData { fake_data: 1 };
        let data1 = TestData { fake_data: 2 };
        let data2 = TestData { fake_data: 3 };

        let root_node: TreeNode<TestData> = Tree::create_root_node(root_data.clone());
        let mut tree: Tree<TreeNode<TestData>> = Tree::create_tree(&root_node);

        let node1 = tree.create_node(data1.clone(), &root_node);
        let node2 = tree.create_node(data2.clone(), &root_node);
        let node1_id = tree.add_node(&node1);
        let node2_id = tree.add_node(&node2);

        let node1_to_root = node1.get_path_to_root();
        let node2_to_root = node2.get_path_to_root();

        assert_eq!(node1_to_root.len(), 2);
        assert_eq!(node2_to_root.len(), 2);

        assert_eq!(node1_to_root[0].data, data1);
        assert_eq!(node1_to_root[1].data, root_data);

        assert_eq!(node2_to_root[0].data, data2);
        assert_eq!(node2_to_root[1].data, root_data);
    }

    // todo - more tests
}
