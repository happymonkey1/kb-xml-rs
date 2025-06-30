use std::collections::HashMap;
use crate::error::ParseError;
use crate::node::{XmlElement, XmlNode};
use crate::parser::document::LexedXmlDocument;
use crate::parser::node::LexedXmlNode;
use crate::parser::parser::KbXmlParser;

pub struct XmlDocument {
    root: Option<XmlNode>
}

impl XmlDocument {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn new_from_root(root: XmlNode) -> Self {
        Self { root: Some(root) }
    }

    pub fn parse(data: &str) -> crate::error::Result<Self> {
        let mut parser = KbXmlParser::new();
        let internal_doc = parser.parse(data)?;
        Ok(XmlDocument::try_from(internal_doc)?)
    }

    pub fn root(&self) -> Option<&XmlNode> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut XmlNode> {
        self.root.as_mut()
    }
}

impl TryFrom<LexedXmlDocument> for XmlDocument {
    type Error = ParseError;
    
    fn try_from(value: LexedXmlDocument) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            return Ok(XmlDocument::new());
        }

        let mut stack: Vec<XmlElement> = Vec::new();
        let mut root: Option<XmlElement> = None;

        for lexed in value.into_iter() {
            match lexed {
                LexedXmlNode::TagOpen { name, namespace } => {
                    let el = XmlElement::new(name, namespace, HashMap::new(), Vec::new());
                    stack.push(el);
                }

                LexedXmlNode::TagSelfClosing { name, namespace } => {
                    let el = XmlElement::new(name, namespace, HashMap::new(), Vec::new());
                    if let Some(parent) = stack.last_mut() {
                        parent.add_element(el);
                    } else if root.is_none() {
                        root = Some(el);
                    } else {
                        return Err(ParseError::UnexpectedRoot);
                    }
                }
                
                LexedXmlNode::Attribute { key, value } => {
                    if let Some(parent) = stack.last_mut() {
                        let value = if let Some(value) = value {
                            value
                        } else {
                            String::new()
                        };
                        
                        parent.set_attr(key, value);
                    } else {
                        return Err(ParseError::InvalidParent)
                    }
                }

                LexedXmlNode::Content(text) => {
                    if let Some(parent) = stack.last_mut() {
                        parent.add_text(&text);
                    } else {
                        return Err(ParseError::ContentOutsideRoot);
                    }
                }

                LexedXmlNode::TagClose { name, namespace } => {
                    let closed = stack.pop().ok_or_else(|| ParseError::UnmatchedCloseTag(name.clone()))?;

                    let names_mismatch = closed.name() != &name || closed.namespace() != namespace.as_ref();
                    let is_empty = name.is_empty() && namespace.is_none();
                    if names_mismatch && !is_empty {
                        return Err(ParseError::TagMismatch {
                            expected: (closed.name().clone(), closed.namespace().cloned()),
                            found: (name, namespace),
                        });
                    }

                    if let Some(parent) = stack.last_mut() {
                        parent.add_element(closed);
                    } else if root.is_none() {
                        root = Some(closed);
                    } else {
                        return Err(ParseError::UnexpectedRoot);
                    }
                }
            }
        }

        if !stack.is_empty() {
            return Err(ParseError::UnclosedTags(stack.into_iter().map(|e| e.name().clone()).collect()));
        }

        let root_node = XmlNode::Element(root.ok_or(ParseError::EmptyDocument)?);
        Ok(XmlDocument::new_from_root(root_node))
    }
}

#[cfg(test)]
mod tests {
    use crate::document::XmlDocument;
    use crate::error::{ParseError, Result};
    use crate::node::XmlNode;

    #[test]
    fn when_parse_basic_document_then_succeed() -> Result<()> {
        let doc = XmlDocument::parse(r#"
            <book xmlns="https://rs.kablunk.com">
                <title>Kablunk</title>
                <author>happymonkey1</author>
            </book>
        "#)?;
        
        assert!(doc.root.is_some(), "Root node is not parsed");
        
        let root_node = doc.root.unwrap();
        match &root_node {
            XmlNode::Element(element) => {
                assert_eq!(element.name(), "book");
                let xmlns_attr = element.attr("xmlns").expect("xmlns attribute is present");
                assert_eq!(xmlns_attr.value(), "https://rs.kablunk.com");
            }
            other_node => assert!(false, "Unexpected root node: {other_node:?}")
        }
        
        let root_children = match &root_node {
            XmlNode::Element(element) => {
                element.children()
            }
            other => {
                assert!(false, "Unexpected node while retrieving children: {other:?}");
                // Needed to satisfy the compiler
                return Err(ParseError::InvalidDocumentError)
            }
        };
        assert_eq!(
            root_children.len(),
            2,
            "Expected root to have 2 children, found {} instead",
            root_children.len(),
        );
        
        let first_child_node = &root_children[0];
        let first_child_element = match first_child_node {
            XmlNode::Element(element) => {
                assert_eq!(element.name(), "title");
                element
            }
            other_node => {
                assert!(false, "Unexpected node while checking first child: {other_node:?}");
                // Needed to satisfy the compiler
                return Err(ParseError::InvalidDocumentError)
            }
        };

        assert_eq!(first_child_element.children().len(), 1);
        let title_child_node = first_child_element.children().first().expect("Title has child");
        match title_child_node {
            XmlNode::Text(text) => {
                assert_eq!(text, "Kablunk")
            }
            other_node =>
                assert!(false, "Unexpected node while checking title children: {other_node:?}")
        }

        let second_child_node = &root_children[1];
        let second_child_element = match second_child_node {
            XmlNode::Element(element) => {
                assert_eq!(element.name(), "author");
                element
            }
            other_node => {
                assert!(false, "Unexpected node while checking first child: {other_node:?}");
                // Needed to satisfy the compiler
                return Err(ParseError::InvalidDocumentError)
            }
        };
        
        assert_eq!(second_child_element.children().len(), 1);
        let author_child_node = second_child_element.children().first().expect("Author has child");
        match author_child_node {
            XmlNode::Text(text) => {
                assert_eq!(text, "happymonkey1")
            }
            other_node =>
                assert!(false, "Unexpected node while checking author children: {other_node:?}")
        }
        
        
        Ok(())
    }

    #[test]
    fn when_parse_multiple_attribute_name_and_value_then_succeed() -> Result<()> {
        let mut doc = XmlDocument::parse("<data foo=\"bar\" baz=\"qux\" />")?;

        let root = doc.root_mut()
            .expect("Root is valid")
            .as_element_mut()
            .expect("Root is element");

        let attributes = root.attributes_mut();
        assert_eq!(attributes.len(), 2);

        let first_attribute = attributes.get("foo");
        assert!(first_attribute.is_some(), "Foo attribute was not parsed");
        assert_eq!(first_attribute.expect("Foo attribute parsed"), "bar");

        let second_attribute = attributes.get("baz");
        assert!(second_attribute.is_some(), "Baz attribute was not parsed");
        assert_eq!(second_attribute.expect("Baz attribute parsed"), "qux");

        Ok(())
    }
    
    #[test]
    fn when_parse_sample_file_then_succeed() -> Result<()> {
        XmlDocument::parse(include_str!("../resources/book.xml"))?;
        
        Ok(())
    }
    
}