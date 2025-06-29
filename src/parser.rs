use crate::error::{ParseError, Result};

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

}

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

#[derive(Default)]
pub struct XmlDocument {
    nodes: Vec<XmlNode>,
}

impl XmlDocument {
    pub fn new() -> Self {

        Self {
            nodes: Vec::<XmlNode>::new(),
        }
    }
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
        self.document.nodes.push(xml_node);
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{KbXmlParser, XmlNode};

    #[test]
    fn when_parse_single_tag_then_succeed() {
        let data = "<hello>hi</hello>";

        let mut parser = KbXmlParser::new();

        let doc = parser.parse(data.to_string());
        assert!(doc.is_ok());

        let doc = doc.unwrap();
        assert_eq!(doc.nodes.len(), 2);

        let open_node = doc.nodes[0].clone();
        let expected_open_node_name = String::from("hello");
        assert!(matches!(open_node, XmlNode::TagOpen { .. }));
        match open_node {
            XmlNode::TagOpen { name, namespace } => {
                assert_eq!(name, expected_open_node_name);
                assert_eq!(namespace, None);
            }
            other => assert!(false, "Expected XmlNode::TagOpen, found: {other:?}")
        }

        let close_node = doc.nodes[1].clone();
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

}
