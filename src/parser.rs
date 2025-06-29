use crate::document::XmlDocument;
use crate::error::{ParseError, Result};
use crate::node::XmlNode;

enum Token {
    Space,
    LessThan,
    GreaterThan,
    Slash,
    Colon,
    Char,
}

impl Token {
    pub fn char_to_token(ch: char) -> Token {
        match ch {
            ' ' | '\n' | '\t' | '\r' => Token::Space,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            '/' => Token::Slash,
            ':' => Token::Colon,
            _ => Token::Char,
        }
    }
}

enum State {
    Data,
    TagBegin,
    TagName,
    TagEnd,
}

pub struct KbXmlParser {
    current_state: State,
    data_buffer: String,
    tag_name: String,
    tag_namespace: String,
    is_closing: bool,
    document: XmlDocument,
}

impl KbXmlParser {

    pub fn new() -> Self {
        Self {
            current_state: State::Data,
            data_buffer: String::new(),
            tag_name: String::new(),
            tag_namespace: String::new(),
            is_closing: false,
            document: XmlDocument::new(),
        }
    }

    pub fn parse(&mut self, data: String) -> Result<XmlDocument> {
        for ch in data.chars() {
            self.step(ch)?
        }

        let doc = std::mem::take(&mut self.document);
        Ok(doc)
    }

    fn step(&mut self, ch: char) -> Result<()> {
        let tok = Token::char_to_token(ch);
        let next_state: State = match &self.current_state {
            State::Data =>
                match tok {
                    Token::LessThan => {
                        let trimmed_buffer  = self.data_buffer.trim();
                        if !trimmed_buffer.is_empty() {
                            self.push_node(XmlNode::new_content(trimmed_buffer.to_string()))
                        }

                        self.tag_name = String::new();
                        self.is_closing = false;

                        State::TagBegin
                    }
                    _ => {
                        self.data_buffer.push(ch);
                        State::Data
                    }
                }
            State::TagBegin =>
                match tok {
                    Token::Char => {
                        self.tag_name.push(ch);

                        State::TagName
                    }
                    Token::GreaterThan => {

                        State::TagEnd
                    }
                    Token::Slash => {
                        self.is_closing = true;

                        State::TagBegin
                    }
                    _ => return Err(ParseError::InvalidStateError)
                }
            State::TagName =>
                match tok {
                    Token::Colon => {
                        std::mem::swap(&mut self.tag_name, &mut self.tag_namespace);

                        State::TagBegin
                    }
                    Token::Char => {
                        self.tag_name.push(ch);

                        State::TagName
                    }
                    Token::GreaterThan => {
                        let name = std::mem::take(&mut self.tag_name);
                        let namespace = if !self.tag_namespace.is_empty() {
                            Some(std::mem::take(&mut self.tag_namespace))
                        } else {
                            None
                        };

                        if self.is_closing {
                            self.push_node(XmlNode::new_tag_close(name, namespace))
                        } else {
                            self.push_node(XmlNode::new_tag_open(name, namespace))
                        }

                        State::Data
                    }
                    _ => return Err(ParseError::InvalidStateError),
                }
            State::TagEnd =>
                match tok {
                    Token::GreaterThan => {
                        let name = std::mem::take(&mut self.tag_name);
                        let namespace = if !self.tag_namespace.is_empty() {
                            Some(std::mem::take(&mut self.tag_namespace))
                        } else {
                            None
                        };

                        self.push_node(XmlNode::new_tag_close(name, namespace));

                        State::Data
                    }
                    Token::Char => {
                        /* no-op */

                        State::TagEnd
                    }
                    _ => return Err(ParseError::InvalidStateError)
                }
        };

        self.current_state = next_state;
        Ok(())
    }

    fn push_node(&mut self, xml_node: XmlNode) {
        self.document.push_node(xml_node);
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{KbXmlParser, XmlNode};
    use crate::error::{Result};

    #[test]
    fn when_parse_single_tag_then_succeed() {
        let data = "<hello>hi</hello>";

        let mut parser = KbXmlParser::new();

        let doc = parser.parse(data.to_string());
        assert!(doc.is_ok());

        let doc = doc.unwrap();
        assert_eq!(doc.len(), 3);

        let open_node = doc.get_node_at(0).cloned().unwrap();
        let expected_open_node_name = String::from("hello");
        assert!(matches!(open_node, XmlNode::TagOpen { .. }));
        match open_node {
            XmlNode::TagOpen { name, namespace } => {
                assert_eq!(name, expected_open_node_name);
                assert_eq!(namespace, None);
            }
            other => assert!(false, "Expected XmlNode::TagOpen, found: {other:?}")
        }

        let content_node = doc.get_node_at(1).cloned().unwrap();
        let expected_content = "hi".to_string();
        assert!(matches!(content_node, XmlNode::Content { .. }));
        match content_node {
            XmlNode::Content(data) => {
                assert_eq!(data, expected_content);
            }
            other => assert!(false, "Expected XmlNode::Content, found: {other:?}")
        }

        let close_node = doc.get_node_at(2).cloned().unwrap();
        let expected_close_node_name = String::from("hello");
        assert!(matches!(close_node, XmlNode::TagClose { .. }));
        match close_node {
            XmlNode::TagClose { name, namespace } => {
                assert_eq!(name, expected_close_node_name);
                assert_eq!(namespace, None);
            }
            other => assert!(false, "Expected XmlNode::TagClose, found: {other:?}")
        }
    }

    #[test]
    fn when_parse_node_with_namespace_then_succeed() -> Result<()> {
        let data = "<kb:hello></kb:hello>";

        let mut parser = KbXmlParser::new();
        let doc = parser.parse(data.to_string())?;


        let open_node = doc.get_node_at(0).cloned().unwrap();
        let expected_tag_name = "hello".to_string();
        let expected_tag_namespace = "kb".to_string();

        match open_node {
            XmlNode::TagOpen { name, namespace } => {
                assert_eq!(name, expected_tag_name);

                assert!(namespace.is_some(), "Namespace can not be empty");
                assert_eq!(namespace.unwrap(), expected_tag_namespace);
            }
            other => assert!(false, "Expected XmlNode::TagOpen, found: {other:?}")
        }

        let close_node = doc.get_node_at(1).cloned().unwrap();
        match close_node {
            XmlNode::TagClose { name, namespace } => {
                assert_eq!(name, expected_tag_name);

                assert!(namespace.is_some(), "Namespace can not be empty");
                assert_eq!(namespace.unwrap(), expected_tag_namespace);
            }
            other => assert!(false, "Expected XmlNode::TagClose, found: {other:?}")
        }

        Ok(())
    }

}
