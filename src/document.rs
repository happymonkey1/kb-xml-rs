use crate::node::XmlNode;

#[derive(Debug, Default)]
pub struct XmlDocument {
    nodes: Vec<XmlNode>,
}

impl XmlDocument {
    pub fn new() -> Self {
        Self {
            nodes: Vec::<XmlNode>::new(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, XmlNode> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, XmlNode> {
        self.nodes.iter_mut()
    }

    pub fn into_iter(self) -> std::vec::IntoIter<XmlNode> {
        self.nodes.into_iter()
    }
    
    pub fn push_node(&mut self, node: XmlNode) {
        self.nodes.push(node)
    }
    
    pub fn get_nodes(&self) -> &Vec<XmlNode> { 
        &self.nodes
    }
    
    /// Returns the number of nodes in the [XmlDocument]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn get_node_at(&self, index: usize) -> Option<&XmlNode> {
        self.nodes.get(index)
    }
    
    pub fn get_node_at_mut(&mut self, index: usize) -> Option<&mut XmlNode> {
        self.nodes.get_mut(index)
    }
}