use std::collections::HashMap;

#[derive(Debug)]
pub enum XmlNode {
    Element(XmlElement),
    Text(String),
    Comment(String),
    CData(String),
}

#[derive(Debug)]
pub struct XmlElement {
    name: String,
    namespace: Option<String>,
    attributes: HashMap<String, String>,
    children: Vec<XmlNode>,
}

pub struct XmlAttribute<'a> {
    key: &'a str,
    value: &'a str,
}

impl XmlNode {
    pub fn new_element(
        name: String,
        namespace: Option<String>,
        attributes: HashMap<String, String>,
        children: Vec<XmlNode>,
    ) -> Self {
        Self::Element(XmlElement::new(name, namespace, attributes, children))
    }

    pub fn new_text(text: String) -> Self {
        Self::Text(text)
    }

    pub fn new_comment(comment: String) -> Self {
        Self::Comment(comment)
    }

    pub fn new_cdata(data: String) -> Self {
        Self::CData(data)
    }
}

// Querying API
impl XmlElement {
    pub fn new(
        name: String,
        namespace: Option<String>,
        attributes: HashMap<String, String>,
        children: Vec<XmlNode>,
    ) -> Self {
        Self {
            name,
            namespace,
            attributes,
            children
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn namespace(&self) -> Option<&String> {
        self.namespace.as_ref()
    }

    pub fn attr<'a>(&'a self, key: &'a str) -> Option<XmlAttribute<'a>> {
        if self.attributes.contains_key(key) {
            let attr = self.attributes.get(key).unwrap();
            Some(XmlAttribute {
                key,
                value: attr.as_str(),
            })
        } else {
            None
        }
    }

    pub fn children(&self) -> &[XmlNode] {
        self.children.as_slice()
    }

    pub fn children_mut(&mut self) -> &mut [XmlNode] {
        self.children.as_mut_slice()
    }

    pub fn elements(&self) -> impl Iterator<Item = &XmlElement> {
        self.children.iter()
            .filter_map(|node| {
                match node {
                    XmlNode::Element(element) => Some(element),
                    _ => None,
                }
            })
    }

    pub fn elements_mut(&mut self) -> impl Iterator<Item = &mut XmlElement> {
        self.children.iter_mut()
            .filter_map(|node| {
                match node {
                    XmlNode::Element(element) => Some(element),
                    _ => None,
                }
            })
    }

    pub fn first_named_child(&self, name: &str) -> Option<&XmlElement> {
        self.children.iter()
            .find_map(|node| {
                match node {
                    XmlNode::Element(element) => {
                        if element.name.as_str() == name {
                            Some(element)
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            })
    }
}

// Mutation API
impl XmlElement {
    pub fn set_attr(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn add_text(&mut self, text: &str) {
        self.children.push(XmlNode::new_text(text.to_string()))
    }

    pub fn add_child(&mut self, child: XmlNode) {
        self.children.push(child)
    }

    pub fn add_element(&mut self, element: XmlElement) {
        self.children.push(XmlNode::Element(element))
    }
}

impl <'a> XmlAttribute<'a> {
    pub fn key(&self) -> &'a str {
        self.key
    }
    
    pub fn value(&self) -> &'a str {
        self.value
    }
}