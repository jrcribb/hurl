/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2025 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use std::fmt;

use crate::ast::JsonValue;
use crate::reader::Pos;
use crate::typing::{SourceString, ToSource};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyValue {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub key: Template,
    pub space1: Whitespace,
    pub space2: Whitespace,
    pub value: Template,
    pub line_terminator0: LineTerminator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultilineString {
    pub kind: MultilineStringKind,
    pub attributes: Vec<MultilineStringAttribute>,
}

impl fmt::Display for MultilineString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body = match &self.kind {
            MultilineStringKind::Text(text)
            | MultilineStringKind::Json(text)
            | MultilineStringKind::Xml(text) => text.value.to_string(),
            MultilineStringKind::GraphQl(graphql) => {
                let var = match &graphql.variables {
                    None => String::new(),
                    Some(var) => {
                        format!(
                            "variables{}{}{}",
                            var.space.value, var.value, var.whitespace.value
                        )
                    }
                };
                format!("{}{}", graphql.value, var)
            }
        };
        write!(f, "{body}")
    }
}

impl MultilineString {
    pub fn lang(&self) -> &'static str {
        match self.kind {
            MultilineStringKind::Text(_) => "",
            MultilineStringKind::Json(_) => "json",
            MultilineStringKind::Xml(_) => "xml",
            MultilineStringKind::GraphQl(_) => "graphql",
        }
    }

    pub fn value(&self) -> Template {
        match &self.kind {
            MultilineStringKind::Text(text)
            | MultilineStringKind::Json(text)
            | MultilineStringKind::Xml(text) => text.value.clone(),
            MultilineStringKind::GraphQl(text) => text.value.clone(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultilineStringKind {
    Text(Text),
    Json(Text),
    Xml(Text),
    GraphQl(GraphQl),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultilineStringAttribute {
    Escape,
    NoVariable,
}

impl fmt::Display for MultilineStringAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultilineStringAttribute::Escape => write!(f, "escape"),
            MultilineStringAttribute::NoVariable => write!(f, "novariable"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    pub space: Whitespace,
    pub newline: Whitespace,
    pub value: Template,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphQl {
    pub space: Whitespace,
    pub newline: Whitespace,
    pub value: Template,
    pub variables: Option<GraphQlVariables>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphQlVariables {
    pub space: Whitespace,
    pub value: JsonValue,
    pub whitespace: Whitespace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base64 {
    pub space0: Whitespace,
    pub value: Vec<u8>,
    pub source: SourceString,
    pub space1: Whitespace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    pub space0: Whitespace,
    pub filename: Template,
    pub space1: Whitespace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub delimiter: Option<char>,
    pub elements: Vec<TemplateElement>,
    pub source_info: SourceInfo,
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buffer = String::new();
        for element in self.elements.iter() {
            buffer.push_str(element.to_string().as_str());
        }
        write!(f, "{buffer}")
    }
}

impl ToSource for Template {
    fn to_source(&self) -> SourceString {
        let mut s = SourceString::new();
        if let Some(d) = self.delimiter {
            s.push(d);
        }
        let elements: Vec<SourceString> = self.elements.iter().map(|e| e.to_source()).collect();
        s.push_str(elements.join("").as_str());
        if let Some(d) = self.delimiter {
            s.push(d);
        }
        s
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateElement {
    String { value: String, source: SourceString },
    Placeholder(Placeholder),
}

impl fmt::Display for TemplateElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TemplateElement::String { value, .. } => value.clone(),
            TemplateElement::Placeholder(value) => format!("{{{{{value}}}}}"),
        };
        write!(f, "{s}")
    }
}

impl ToSource for TemplateElement {
    fn to_source(&self) -> SourceString {
        match self {
            TemplateElement::String { source, .. } => source.clone(),
            TemplateElement::Placeholder(value) => format!("{{{{{value}}}}}").to_source(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub value: String,
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Whitespace {
    pub value: String,
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Number {
    Float(Float),
    Integer(I64),
    BigInteger(String),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Number::Float(value) => write!(f, "{}", value),
            Number::Integer(value) => write!(f, "{}", value),
            Number::BigInteger(value) => write!(f, "{}", value),
        }
    }
}

impl ToSource for Number {
    fn to_source(&self) -> SourceString {
        match self {
            Number::Float(value) => value.to_source(),
            Number::Integer(value) => value.to_source(),
            Number::BigInteger(value) => value.to_source(),
        }
    }
}

// keep Number terminology for both Integer and Decimal Numbers
// different representation for the same float value
// 1.01 and 1.010

#[derive(Clone, Debug)]
pub struct Float {
    value: f64,
    source: SourceString,
}

impl Float {
    pub fn new(value: f64, source: SourceString) -> Float {
        Float { value, source }
    }

    pub fn as_f64(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ToSource for Float {
    fn to_source(&self) -> SourceString {
        self.source.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct U64 {
    value: u64,
    source: SourceString,
}

impl U64 {
    pub fn new(value: u64, source: SourceString) -> U64 {
        U64 { value, source }
    }

    pub fn as_u64(&self) -> u64 {
        self.value
    }
}

impl fmt::Display for U64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ToSource for U64 {
    fn to_source(&self) -> SourceString {
        self.source.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct I64 {
    value: i64,
    source: SourceString,
}

impl I64 {
    pub fn new(value: i64, source: SourceString) -> I64 {
        I64 { value, source }
    }

    pub fn as_i64(&self) -> i64 {
        self.value
    }
}

impl fmt::Display for I64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ToSource for I64 {
    fn to_source(&self) -> SourceString {
        self.source.clone()
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for Float {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineTerminator {
    pub space0: Whitespace,
    pub comment: Option<Comment>,
    pub newline: Whitespace,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bytes {
    Json(JsonValue),
    Xml(String),
    MultilineString(MultilineString),
    OnelineString(Template),
    Base64(Base64),
    File(File),
    Hex(Hex),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hex {
    pub space0: Whitespace,
    pub value: Vec<u8>,
    pub source: SourceString,
    pub space1: Whitespace,
}

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "hex,{}{}{};",
            self.space0.value, self.source, self.space1.value
        )
    }
}

/// Literal Regex.
#[derive(Clone, Debug)]
pub struct Regex {
    pub inner: regex::Regex,
}

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl PartialEq for Regex {
    fn eq(&self, other: &Self) -> bool {
        self.inner.to_string() == other.inner.to_string()
    }
}

impl Eq for Regex {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SourceInfo {
    pub start: Pos,
    pub end: Pos,
}

impl SourceInfo {
    pub fn new(start: Pos, end: Pos) -> SourceInfo {
        SourceInfo { start, end }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placeholder {
    pub space0: Whitespace,
    pub expr: Expr,
    pub space1: Whitespace,
}

impl fmt::Display for Placeholder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expr)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expr {
    pub source_info: SourceInfo,
    pub kind: ExprKind,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprKind {
    Variable(Variable),
    Function(Function),
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExprKind::Variable(variable) => write!(f, "{}", variable),
            ExprKind::Function(function) => write!(f, "{}", function),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub source_info: SourceInfo,
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Function {
    NewDate,
    NewUuid,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::NewDate => write!(f, "newDate"),
            Function::NewUuid => write!(f, "newUuid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::json::{JsonListElement, JsonObjectElement, JsonValue};
    use crate::typing::ToSource;

    #[test]
    fn test_float() {
        assert_eq!(
            Float {
                value: 1.0,
                source: "1.0".to_source()
            }
            .to_source()
            .as_str(),
            "1.0"
        );
        assert_eq!(
            Float {
                value: 1.0,
                source: "1.0".to_source()
            }
            .to_string(),
            "1"
        );

        assert_eq!(
            Float {
                value: 1.01,
                source: "1.01".to_source()
            }
            .to_source()
            .as_str(),
            "1.01"
        );
        assert_eq!(
            Float {
                value: 1.01,
                source: "1.01".to_source()
            }
            .to_string(),
            "1.01"
        );

        assert_eq!(
            Float {
                value: 1.01,
                source: "1.010".to_source()
            }
            .to_source()
            .as_str(),
            "1.010"
        );
        assert_eq!(
            Float {
                value: 1.01,
                source: "1.010".to_source()
            }
            .to_string(),
            "1.01"
        );

        assert_eq!(
            Float {
                value: -1.333,
                source: "-1.333".to_source()
            }
            .to_source()
            .as_str(),
            "-1.333"
        );
        assert_eq!(
            Float {
                value: -1.333,
                source: "-1.333".to_source()
            }
            .to_string(),
            "-1.333"
        );
    }

    fn whitespace() -> Whitespace {
        Whitespace {
            value: String::new(),
            source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
        }
    }

    fn variable_placeholder() -> Placeholder {
        Placeholder {
            space0: whitespace(),
            expr: Expr {
                kind: ExprKind::Variable(Variable {
                    name: "name".to_string(),
                    source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
                }),
                source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
            },
            space1: whitespace(),
        }
    }

    fn hello_template() -> Template {
        Template {
            delimiter: None,
            elements: vec![
                TemplateElement::String {
                    value: "Hello ".to_string(),
                    source: "Hello ".to_source(),
                },
                TemplateElement::Placeholder(variable_placeholder()),
                TemplateElement::String {
                    value: "!".to_string(),
                    source: "!".to_source(),
                },
            ],
            source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
        }
    }

    #[test]
    fn test_template() {
        assert_eq!(hello_template().to_string(), "Hello {{name}}!");
    }

    #[test]
    fn test_template_to_string() {
        assert_eq!(
            "{{x}}".to_string(),
            JsonValue::Placeholder(Placeholder {
                space0: Whitespace {
                    value: String::new(),
                    source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
                },
                expr: Expr {
                    kind: ExprKind::Variable(Variable {
                        name: "x".to_string(),
                        source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
                    }),
                    source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                },
                space1: Whitespace {
                    value: String::new(),
                    source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
                },
            })
            .to_string()
        );
        assert_eq!(
            "1".to_string(),
            JsonValue::Number("1".to_string()).to_string()
        );
        assert_eq!(
            "\"hello\"".to_string(),
            JsonValue::String(Template {
                delimiter: None,
                elements: vec![TemplateElement::String {
                    value: "hello".to_string(),
                    source: "hello".to_source(),
                }],
                source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
            })
            .to_string()
        );
        assert_eq!("true".to_string(), JsonValue::Boolean(true).to_string());
        assert_eq!(
            "[]".to_string(),
            JsonValue::List {
                space0: String::new(),
                elements: vec![],
            }
            .to_string()
        );
        assert_eq!(
            "[1, 2, 3]".to_string(),
            JsonValue::List {
                space0: String::new(),
                elements: vec![
                    JsonListElement {
                        space0: String::new(),
                        value: JsonValue::Number("1".to_string()),
                        space1: String::new(),
                    },
                    JsonListElement {
                        space0: " ".to_string(),
                        value: JsonValue::Number("2".to_string()),
                        space1: String::new(),
                    },
                    JsonListElement {
                        space0: " ".to_string(),
                        value: JsonValue::Number("3".to_string()),
                        space1: String::new(),
                    }
                ],
            }
            .to_string()
        );
        assert_eq!(
            "{}".to_string(),
            JsonValue::Object {
                space0: String::new(),
                elements: vec![],
            }
            .to_string()
        );
        assert_eq!(
            "{ \"id\": 123 }".to_string(),
            JsonValue::Object {
                space0: String::new(),
                elements: vec![JsonObjectElement {
                    space0: " ".to_string(),
                    name: Template {
                        delimiter: Some('"'),
                        elements: vec![TemplateElement::String {
                            value: "id".to_string(),
                            source: "id".to_source(),
                        }],
                        source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                    },
                    space1: String::new(),
                    space2: " ".to_string(),
                    value: JsonValue::Number("123".to_string()),
                    space3: " ".to_string(),
                }],
            }
            .to_string()
        );
        assert_eq!("null".to_string(), JsonValue::Null.to_string());
    }

    #[test]
    fn test_template_to_source() {
        assert_eq!(
            TemplateElement::Placeholder(Placeholder {
                space0: Whitespace {
                    value: String::new(),
                    source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                },
                expr: Expr {
                    kind: ExprKind::Variable(Variable {
                        name: "name".to_string(),
                        source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                    }),
                    source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                },
                space1: Whitespace {
                    value: String::new(),
                    source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                },
            })
            .to_source(),
            "{{name}}".to_source()
        );
        assert_eq!(
            Template {
                delimiter: None,
                elements: vec![TemplateElement::Placeholder(Placeholder {
                    space0: Whitespace {
                        value: String::new(),
                        source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                    },
                    expr: Expr {
                        kind: ExprKind::Variable(Variable {
                            name: "name".to_string(),
                            source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                        }),
                        source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                    },
                    space1: Whitespace {
                        value: String::new(),
                        source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
                    },
                })],
                source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
            }
            .to_source(),
            "{{name}}".to_source()
        );
    }
}
