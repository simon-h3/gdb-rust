/*
    Simon H - 2024
*/

pub const PATH: &str = "database/test_database.db"; // The path to the database
pub const EXPORT_PATH: &str = "database/output.json"; // The path to the exported database
pub const INPUT_PATH: &str = "database/input.txt"; // Input file path, for testing
pub const RLT_PAD: usize = 7; // Relationship padding
pub const ATR_PAD: usize = 2; // Attribute padding

use serde_derive::{Deserialize, Serialize};
use std::mem::size_of;

// Define the structs used in the database...

// BlockType enum
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    Empty,
    Unset,
    Node,
    Relationship,
    Attribute,
    Final,
}

// Default is used to set the default value of a struct (when defining empty struct)
impl Default for BlockType {
    fn default() -> Self {
        BlockType::Unset // Default value of BlockType is Empty
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Header {
    pub total_blocks: u64,
    pub first_empty: u64,
    pub db_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)] // This is used to make sure the struct is represented in memory the same way as in C
pub struct NodeBlock {
    pub block_type: BlockType,
    pub node: Node,
}

impl Default for NodeBlock {
    fn default() -> Self {
        NodeBlock {
            block_type: BlockType::Empty,
            node: {
                Node {
                    id: 0,
                    name: [
                        'E', 'M', 'P', 'T', 'Y', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
                        '\0', '\0', '\0',
                    ],
                    rlt_head: 0,
                    attr_head: 0,
                }
            },
        }
    }
}

pub struct TestSize {
    pub blocks: [u8; 64],
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Node {
    pub id: u64,
    pub name: [char; 16],
    pub rlt_head: u64,
    pub attr_head: u64,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Block {
    pub block_type: BlockType,
    pub pad: [u64; 11],
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(C)]
pub struct Relationship {
    pub node_from: u64,
    pub node_to: u64,
    pub rlt_next: u64,
    pub attr_head: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct RelationshipBlock {
    pub block_type: BlockType,
    pub relationship: Relationship,
    pub pad: [u64; RLT_PAD],
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(C)]
pub struct Attribute {
    pub value: [char; 16],
    pub attr_next: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct AttributeBlock {
    pub block_type: BlockType,
    pub attribute: Attribute,
    pub pad: [u64; ATR_PAD],
}

// Define a public function that uses the structs
pub fn print_struct_info() {
    println!("Test Struct Info:     {}\r", size_of::<TestSize>());
    println!("Header Size:          {}\r", size_of::<Header>());
    // println!("Block Size:           {}\r", size_of::<NodeBlock>());
    println!("Node Size:            {}\r", size_of::<Node>());
    println!("Relationship Size:    {}\r", size_of::<Relationship>());
    println!("Attribute Size:       {}\r", size_of::<Attribute>());
    println!("NodeBlock Size:       {}\r", size_of::<NodeBlock>());
    println!("Relt Block Size:      {}\r", size_of::<RelationshipBlock>());
    println!("AttributeBlock Size:  {}\r", size_of::<AttributeBlock>());
    println!("Generic Block Size:   {}\r", size_of::<Block>());
    println!("String Size:          {}\r", size_of::<String>());
    println!("----------------------");
}

pub fn assert_struct_size_equality() {
    // let SIZE: usize = size_of::<Header>();
    let SIZE: usize = size_of::<NodeBlock>();
    assert_eq!(size_of::<NodeBlock>(), SIZE);
    assert_eq!(size_of::<RelationshipBlock>(), SIZE);
    assert_eq!(size_of::<AttributeBlock>(), SIZE);
    assert_eq!(size_of::<Block>(), SIZE);
}
