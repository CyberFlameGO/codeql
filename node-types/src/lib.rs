use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

use std::collections::BTreeSet as Set;
use std::fs;

#[derive(Debug)]
pub enum Entry {
    Union {
        type_name: TypeName,
        members: Set<TypeName>,
    },
    Table {
        type_name: TypeName,
        fields: Vec<Field>,
    },
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct TypeName {
    pub kind: String,
    pub named: bool,
}

#[derive(Debug)]
pub struct Field {
    pub parent: TypeName,
    pub types: Set<TypeName>,
    /// The name of the field or None for the anonymous 'children'
    /// entry from node_types.json
    pub name: Option<String>,
    pub storage: Storage,
}

#[derive(Debug)]
pub enum Storage {
    /// the field is stored as a column in the parent table
    Column,
    // the field is store in a link table
    Table {
        index: usize,
    },
}

pub fn read_node_types(node_types_path: &Path) -> std::io::Result<Vec<Entry>> {
    let file = fs::File::open(node_types_path)?;
    let node_types = serde_json::from_reader(file)?;
    Ok(convert_nodes(node_types))
}

fn convert_type(node_type: &NodeType) -> TypeName {
    TypeName {
        kind: node_type.kind.to_string(),
        named: node_type.named,
    }
}

fn convert_types(node_types: &Vec<NodeType>) -> Set<TypeName> {
    let iter = node_types.iter().map(convert_type).collect();
    std::collections::BTreeSet::from(iter)
}
pub fn convert_nodes(nodes: Vec<NodeInfo>) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

    for node in nodes {
        if let Some(subtypes) = &node.subtypes {
            // It's a tree-sitter supertype node, for which we create a union
            // type.
            entries.push(Entry::Union {
                type_name: TypeName {
                    kind: node.kind,
                    named: node.named,
                },
                members: convert_types(&subtypes),
            });
        } else {
            // It's a product type, defined by a table.
            let type_name = TypeName {
                kind: node.kind,
                named: node.named,
            };
            let mut fields = Vec::new();

            // If the type also has fields or children, then we create either
            // auxiliary tables or columns in the defining table for them.
            if let Some(node_fields) = &node.fields {
                for (field_name, field_info) in node_fields {
                    add_field(
                        &type_name,
                        Some(field_name.to_string()),
                        field_info,
                        &mut fields,
                    );
                }
            }
            if let Some(children) = &node.children {
                // Treat children as if they were a field called 'child'.
                add_field(&type_name, None, children, &mut fields);
            }
            entries.push(Entry::Table { type_name, fields });
        }
    }
    entries
}

fn add_field(
    parent_type_name: &TypeName,
    field_name: Option<String>,
    field_info: &FieldInfo,
    fields: &mut Vec<Field>,
) {
    let storage;
    if !field_info.multiple && field_info.required {
        // This field must appear exactly once, so we add it as
        // a column to the main table for the node type.
        storage = Storage::Column;
    } else {
        // This field can appear zero or multiple times, so put
        // it in an auxiliary table.
        storage = Storage::Table {
            index: fields.len(),
        };
    }
    fields.push(Field {
        parent: TypeName {
            kind: parent_type_name.kind.to_string(),
            named: parent_type_name.named,
        },
        types: convert_types(&field_info.types),
        name: field_name,
        storage,
    });
}
#[derive(Deserialize)]
pub struct NodeInfo {
    #[serde(rename = "type")]
    pub kind: String,
    pub named: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<BTreeMap<String, FieldInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<FieldInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtypes: Option<Vec<NodeType>>,
}

#[derive(Deserialize)]
pub struct NodeType {
    #[serde(rename = "type")]
    pub kind: String,
    pub named: bool,
}

#[derive(Deserialize)]
pub struct FieldInfo {
    pub multiple: bool,
    pub required: bool,
    pub types: Vec<NodeType>,
}

/// Given a tree-sitter node type's (kind, named) pair, returns a single string
/// representing the (unescaped) name we'll use to refer to corresponding QL
/// type.
pub fn node_type_name(kind: &str, named: bool) -> String {
    if named {
        kind.to_string()
    } else {
        format!("{}_unnamed", kind)
    }
}

const RESERVED_KEYWORDS: [&'static str; 14] = [
    "boolean", "case", "date", "float", "int", "key", "of", "order", "ref", "string", "subtype",
    "type", "unique", "varchar",
];

/// Returns a string that's a copy of `name` but suitably escaped to be a valid
/// QL identifier.
pub fn escape_name(name: &str) -> String {
    let mut result = String::new();

    // If there's a leading underscore, replace it with 'underscore_'.
    if let Some(c) = name.chars().next() {
        if c == '_' {
            result.push_str("underscore");
        }
    }
    for c in name.chars() {
        match c {
            '{' => result.push_str("lbrace"),
            '}' => result.push_str("rbrace"),
            '<' => result.push_str("langle"),
            '>' => result.push_str("rangle"),
            '[' => result.push_str("lbracket"),
            ']' => result.push_str("rbracket"),
            '(' => result.push_str("lparen"),
            ')' => result.push_str("rparen"),
            '|' => result.push_str("pipe"),
            '=' => result.push_str("equal"),
            '~' => result.push_str("tilde"),
            '?' => result.push_str("question"),
            '`' => result.push_str("backtick"),
            '^' => result.push_str("caret"),
            '!' => result.push_str("bang"),
            '#' => result.push_str("hash"),
            '%' => result.push_str("percent"),
            '&' => result.push_str("ampersand"),
            '.' => result.push_str("dot"),
            ',' => result.push_str("comma"),
            '/' => result.push_str("slash"),
            ':' => result.push_str("colon"),
            ';' => result.push_str("semicolon"),
            '"' => result.push_str("dquote"),
            '*' => result.push_str("star"),
            '+' => result.push_str("plus"),
            '-' => result.push_str("minus"),
            '@' => result.push_str("at"),
            _ if c.is_uppercase() => {
                result.push_str(&c.to_lowercase().to_string());
                result.push('_')
            }
            _ => result.push(c),
        }
    }

    for &keyword in &RESERVED_KEYWORDS {
        if result == keyword {
            result.push_str("__");
            break;
        }
    }

    result
}