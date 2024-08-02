/*
    Simon H - 2024
*/

use bincode::{deserialize, serialize};
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::mem::size_of;

use crate::attribute::*;
use crate::disk::*;
use crate::relationship::*;

// type imports can be combined, but this is easier to read
use crate::str_conversion;
use crate::types::PATH;
use crate::types::{BlockType, NodeBlock}; // import Block Types
use crate::types::{Header, Node}; // import structs // import db PATH // import fixed static strings helper functions

// custom error macro
macro_rules! custom_error {
    ($msg:expr) => {
        return Err(Error::new(ErrorKind::Other, $msg))
    };
}

// map bincode error to io error
macro_rules! map_bincode_error {
    ($expr:expr) => {
        $expr.map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Bincode serialization error: {:?}", err),
            )
        })
    };
}

//  Given an offset print node to console.
pub fn print_node_name(offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open(PATH).unwrap();

    // Move to offset
    stream.seek(SeekFrom::Start(offset))?;

    // Read bytes into Block struct
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let result_node = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;
    println!("-> {}", str_conversion::char_print(&result_node.node.name));
    Ok(())
}

pub fn compare_node(node1: &Node, node2: &Node) -> bool {
    if node1.id == node2.id {
        return true;
    }
    false
}

//  Given offset, return node structure
pub fn get_node(offset: u64) -> Result<Node> {
    let mut stream = File::open(PATH)?;

    // Rewind the stream to the beginning
    stream.seek(SeekFrom::Start(offset))?;

    // Read the block from the stream
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let deserialized_block = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;

    Ok(deserialized_block.node)
}

//  Create Node and write it to disk
pub fn create_node(new_node: Node) -> Result<()> {
    // let mut stream = File::create(PATH)?;
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // read header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let mut header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    // go to first empty
    stream.seek(SeekFrom::Start(header.first_empty))?;

    let node_block = NodeBlock {
        block_type: BlockType::Node,
        node: new_node,
    };

    // write node information
    let serialized_node_block = map_bincode_error!(serialize(&node_block))?;

    stream.write_all(&serialized_node_block)?;

    // update first empty
    let new_first_empty = get_first_empty(&stream, &header)?;

    // update header
    if new_first_empty == 0 {
        expand_file(10)?;
        // create_node(&new_node);  //TODO: Remove possibility for 0 offset, expand automatically inside new_first_empty
        custom_error!("No first empty found, expanded file.")
    } else {
        println!("New First Empty: {}\r", new_first_empty);
        header.first_empty = new_first_empty;

        let serialized_header = map_bincode_error!(serialize(&header))?;
        stream.seek(SeekFrom::Start(0))?;
        stream.write_all(&serialized_header)?;

        println!(" - Create Node successful...\r\n");
        Ok(())
    }
}

//  Given id, return node Address
pub fn get_node_from_id(id: u64) -> Result<Node> {
    // read header
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header_result = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header_result.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);
        let node = get_node(offset)?;

        if node.id == id {
            return Ok(node);
        }
    }

    custom_error!("Not found, FATAL...");
}

//  Basic Find node function
pub fn get_node_address(node: &Node) -> Result<u64> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);
        let current_node = get_node(offset)?;

        if compare_node(&current_node, &node) {
            return Ok(offset);
        }
    }
    custom_error!("Not found, FATAL...");
}

pub fn get_node_address_from_name(name: &String) -> Result<u64> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;
    let modified_string = str_conversion::str_to_fixed_chars(&name);

    for i in 0..header.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);
        let current_node = get_node(offset)?;

        if current_node.name.eq(&modified_string) {
            // equivalence check (==)
            return Ok(offset);
        }
    }
    custom_error!("Not found, FATAL...");
}

// traverse file and print each block
pub fn print_all_nodes() -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);
        let node = get_node(offset)?;

        println!("Node: {:?}\r", node);
    }

    Ok(())
}

/*
    Modify node's name
*/
pub fn update_node_name(node_address: u64, new_node_name: String) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());

    stream.seek(SeekFrom::Start(node_address))?;
    stream.read_to_end(&mut buffer)?;

    let mut node_block = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;

    node_block.node.name = str_conversion::str_to_fixed_chars(&new_node_name);

    let mut serialized_node_block = map_bincode_error!(serialize(&node_block))?;

    stream.seek(SeekFrom::Start(node_address))?;
    stream.write_all(&serialized_node_block)?;
    Ok(())
}

//  Retrospectively update nodes relationship list head upon creation, if already set follow and set to tail of list.
fn update_node_rlt(mut node: Node, rlt_offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // send a borrowed instance
    let node_address = get_node_address(&node)?;
    stream.seek(SeekFrom::Start(node_address))?;

    if node.rlt_head == 0 {
        node.rlt_head = rlt_offset;

        let serialized_node = map_bincode_error!(serialize(&node))?;
        stream.seek(SeekFrom::Start(node_address))?;
        // stream.write_all_at(&serialized_node, node_address)?;    // linux only command (FileExt)
        stream.write_all(&serialized_node)?;
        println!("Updated Node...");
    } else {
        append_relationship(node_address, rlt_offset)?;
    }

    Ok(())
}

//  Retrospectively update nodes attribute list head upon creation, if already set follow and set to tail of list.
fn update_node_attribute(mut node: Node, attrib_offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let node_address = get_node_address(&node)?;

    if node.attr_head == 0 {
        node.attr_head = attrib_offset;

        let serialized_node = map_bincode_error!(serialize(&node))?;
        stream.seek(SeekFrom::Start(node_address))?;
        // stream.write_all_at(&serialized_node, node_address)?;    // linux only command (FileExt)

        stream.seek(SeekFrom::Start(node_address))?;
        stream.write_all(&serialized_node)?;
    } else {
        append_attribute(node_address, attrib_offset)?;
    }

    Ok(())
}

//  Given a nodes name remove its record
pub fn delete_node_name(name: String) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let node_address = get_node_address_from_name(&name)?;

    // read node information
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;
    let mut node_block = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;

    node_block.block_type = BlockType::Empty;

    // write node information
    let serialized_node_block = map_bincode_error!(serialize(&node_block))?;
    stream.seek(SeekFrom::Start(node_address))?;
    stream.write_all(&serialized_node_block)?;

    Ok(())
}

//  Given a Node remove its record

/*

    Open Nodes File

    Get node address (passed in function arg)

    Seek to node address
    read block into buffer
    deserialise

    set blocktype as BLOCKTYPE::EMPTY

    seek back to node address

    write blocktype

    make call to delete_relations(node.rlt_head)
        return Ok() if relations deleted.

    update header with new first empty (potential check if different)...

*/

pub fn delete_node(node: Node) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let empty_block: NodeBlock = Default::default();
    let empty_block_srl = map_bincode_error!(serialize(&empty_block))?; // serialise

    let node_address = get_node_address(&node)?;

    // read current buffer information
    let mut header_buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut header_buffer)?;
    let mut header = map_bincode_error!(deserialize::<Header>(&header_buffer))?;

    stream.seek(SeekFrom::Start(node_address))?;

    // read node_block
    let mut node_buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut node_buffer)?;
    let mut node_block = map_bincode_error!(deserialize::<NodeBlock>(&node_buffer))?;

    stream.seek(SeekFrom::Start(node_address))?;

    stream.write_all(&empty_block_srl)?;

    if header.first_empty > node_address {
        new_first_empty()?;
    }

    delete_relations(node_block.node.rlt_head)?;
    delete_attributes(node_block.node.attr_head)?;

    Ok(())
}
