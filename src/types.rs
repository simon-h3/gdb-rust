// Define the structs used in the database
pub const PATH: &str = "test_database.db";

use std::mem;

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
#[repr(C)]  // This is used to make sure the struct is represented in memory the same way as in C
pub struct Header {
    pub total_blocks: u64,
    pub first_empty: u64,
    pub db_size: usize,
}

// impl Serialize for Header {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
//         let mut state = serializer.serialize_struct("Header", 3)?;
//         state.serialize_field("total_blocks", &self.total_blocks)?;
//         state.serialize_field("first_empty", &self.first_empty)?;
//         state.serialize_field("db_size", &self.db_size)?;
//         state.end()
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct NodeBlock {
    pub block_type: BlockType,
    pub node: Node,
}

// impl Serialize for NodeBlock {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
//         let mut state = serializer.serialize_struct("NodeBlock", 2)?;
//         state.serialize_field("block_type", &self.block_type)?;
//         state.serialize_field("node", &self.node)?;
//         state.end()
//     }
// }

impl Default for NodeBlock {
    fn default() -> Self {
        NodeBlock {
            block_type: BlockType::Empty,
            node: {
                Node {
                    id: 0,
                    name: "".to_string(),
                    rlt_head: 0,
                    attr_head: 0,
                }
            },
        }
    }
}

// #[derive(Debug, Deserialize)]
// #[repr(C)]
// pub struct EmptyBlock {
//     pub block_type: BlockType,
//     pub node: Node,
// }


#[derive(Default, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub rlt_head: usize,
    pub attr_head: usize,
}

// impl Serialize for Node {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
//         let mut state = serializer.serialize_struct("Node", 4)?;
//         state.serialize_field("id", &self.id)?;
//         state.serialize_field("name", &self.name)?;
//         state.serialize_field("rlt_head", &self.rlt_head)?;
//         state.serialize_field("attr_head", &self.attr_head)?;
//         state.end()
//     }
// }

#[derive(Default, Debug)]
#[repr(C)]
pub struct Relationship {
    pub node_from: usize,
    pub node_to: usize,
    pub rlt_next: usize,
    pub attr_head: usize,
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct Attribute {
    pub value: String,
    pub attr_next: usize,
}


// Define a public function that uses the structs
pub fn print_struct_info() {
    println!("Header Size:          {}\r", mem::size_of::<Header>());
    println!("Block Size:           {}\r", mem::size_of::<NodeBlock>());
    println!("Node Size:            {}\r", mem::size_of::<Node>());
    println!("Relationship Size:    {}\r", mem::size_of::<Relationship>());
    println!("Attribute Size:       {}\r", mem::size_of::<Attribute>());

    println!("String Size:          {}\r", mem::size_of::<String>());
    println!("----------------------");
}