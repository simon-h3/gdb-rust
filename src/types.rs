// Define the structs used in the database
pub const PATH: &str = "database/test_database.db";
pub const EXPORT_PATH: &str = "database/output.json";

use std::mem::size_of;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    Empty,
    Unset,
    Node,
    Relationship,
    Attribute,
}

// Default is used to set the default value of a struct (when defining empty struct)
impl Default for BlockType {
    fn default() -> Self {
        BlockType::Unset    // Default value of BlockType is Empty
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
#[repr(C)]  // This is used to make sure the struct is represented in memory the same way as in C
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
                    name: ".".to_string(),
                    rlt_head: 0,
                    attr_head: 0,
                }
            },
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Node {
    pub id: u64,
    pub name: String,
    pub rlt_head: u64,
    pub attr_head: u64,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Block {
    pub block_type: BlockType,
    pub pad: [u64; 6],

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
    pub pad: [u8; 16],
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Attribute {
    pub value: String,
    pub attr_next: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct AttributeBlock {
    pub block_type: BlockType,
    pub attribute: Attribute,
    pub pad: [u8; 16],
}

// Define a public function that uses the structs
pub fn print_struct_info() {
    println!("Header Size:          {}\r", size_of::<Header>());
    println!("Block Size:           {}\r", size_of::<NodeBlock>());
    println!("Node Size:            {}\r", size_of::<Node>());
    println!("Relationship Size:    {}\r", size_of::<Relationship>());
    println!("Attribute Size:       {}\r", size_of::<Attribute>());
    println!("NodeBlock Size:       {}\r", size_of::<NodeBlock>());
    println!("Relt Block Size:      {}\r", size_of::<RelationshipBlock>());
    println!("AttributeBlock Size:  {}\r", size_of::<AttributeBlock>());
    println!("Generic Block Size:   {}\r", size_of::<Block>()); // !!!!!!!!
    println!("String Size:          {}\r", size_of::<String>());
    println!("----------------------");
}