// TODO: support attributes
#[derive(Clone, Debug)]
pub enum XmlNode {
    TagOpen { name: String, namespace: Option<String>, },
    TagClose { name: String, namespace: Option<String> },
    TagOpenAndClose { name: String, namespace: Option<String>, },

    Content(String),
}

impl XmlNode {

    pub fn new_tag_open(name: String, namespace: Option<String>) -> Self {
        Self::TagOpen { name, namespace }
    }

    pub fn new_tag_close(name: String, namespace: Option<String>) -> Self {
        Self::TagClose { name, namespace }
    }

    pub fn new_tag_open_and_close(name: String, namespace: Option<String>) -> Self {
        Self::TagOpenAndClose { name, namespace }
    }

    pub fn new_content(data: String) -> Self {
        Self::Content(data)
    }

    pub fn get_name(&self) -> Option<&String> {
        match self {
            XmlNode::TagOpen { name, .. } => Some(name),
            XmlNode::TagClose { name, ..} => Some(name),
            XmlNode::TagOpenAndClose { name, .. } => Some(name),
            XmlNode::Content(_) => None,
        }
    }

    pub fn has_namespace(&self) -> bool {
        match self {
            XmlNode::TagOpen { namespace, .. } => namespace.is_some(),
            XmlNode::TagClose { namespace, .. } => namespace.is_some(),
            XmlNode::TagOpenAndClose { namespace, .. } => namespace.is_some(),
            XmlNode::Content(_) => false,
        }
    }

    pub fn get_namespace(&self) -> Option<&String> {
        match self {
            XmlNode::TagOpen { namespace, .. } => namespace.as_ref(),
            XmlNode::TagClose { namespace, ..} => namespace.as_ref(),
            XmlNode::TagOpenAndClose { namespace, .. } => namespace.as_ref(),
            XmlNode::Content(_) => None,
        }
    }

}