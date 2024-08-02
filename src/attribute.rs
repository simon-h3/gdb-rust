/*
    Simon H - 2024
*/

use bincode::{deserialize, serialize};
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::mem::size_of;

// type imports can be combined, but this is easier to read
use crate::disk::*;
use crate::node::*;
// use crate::relationship::*;

use crate::types::{Attribute, Header, ATR_PAD}; // import structs
use crate::types::{AttributeBlock, BlockType, RelationshipBlock};
use crate::types::{NodeBlock, PATH}; // import Block Types

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

pub fn compare_attribute(attrib1: &Attribute, attrib2: &Attribute) -> bool {
    if attrib1.value == attrib2.value {
        return true;
    }
    return false;
}

pub fn get_attribute(offset: u64) -> Result<Attribute> {
    let mut stream = File::open(PATH)?;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<AttributeBlock>());

    stream.seek(SeekFrom::Start(offset))?;
    stream.read_to_end(&mut buffer)?;

    let attribute_block = map_bincode_error!(deserialize::<AttributeBlock>(&buffer))?;

    return Ok(attribute_block.attribute);
}

pub fn create_attribute(new_attribute: Attribute) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // read header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let mut header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    // go to first empty
    stream.seek(SeekFrom::Start(header.first_empty))?;

    let attribute_block = AttributeBlock {
        block_type: BlockType::Attribute,
        attribute: new_attribute,
        pad: [0; ATR_PAD], // pad for consistent sizing across block types
    };

    // write relationship information
    let serialized_attribute_block = map_bincode_error!(serialize(&attribute_block))?;
    stream.write(&serialized_attribute_block)?;

    // update first empty
    let new_first_empty = get_first_empty(&stream, &header)?;

    // update header
    // TODO: update header, update associations...

    Ok(())
}

pub fn get_attribute_address(attribute: &Attribute) -> Result<u64> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // read header
    let mut buffer = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<RelationshipBlock>() as u64);
        let current_attribute = get_attribute(offset)?;

        if compare_attribute(&current_attribute, attribute) {
            return Ok(offset);
        }
    }

    custom_error!("No Relationship Found, FATAL...");
}

//  Print all attributes of a node.
pub fn print_attributes(node_offset: u64) -> Result<()> {
    let node = get_node(node_offset)?;

    // node.attr_head // seek to attr head...

    // TODO: open stream etc... & finish...
    //
    Ok(())
}

pub fn append_attribute(node_address: u64, attribute_offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    stream.seek(SeekFrom::Start(node_address))?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<AttributeBlock>());
    stream.read_to_end(&mut buffer)?;
    let mut attribute_block = map_bincode_error!(deserialize::<AttributeBlock>(&buffer))?;

    if attribute_block.attribute.attr_next == 0 {
        attribute_block.attribute.attr_next = attribute_offset;
        stream.seek(SeekFrom::Start(node_address))?;

        let serialized_attribute_block = map_bincode_error!(serialize(&attribute_block))?;
        stream.write_all(&serialized_attribute_block)?;
        return Ok(());
    } else {
        append_attribute(attribute_block.attribute.attr_next, attribute_offset)?;
    }

    Ok(())
}

// Assigns attribute to EMPTY_BLOCK and writes to disk
pub fn delete_attribute(attribute: Attribute) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let empty_block: NodeBlock = Default::default();
    let empty_block_srl = map_bincode_error!(serialize(&empty_block))?; // serialise

    let attr_address = get_attribute_address(&attribute)?;

    // read current header information
    let mut header_buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut header_buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&header_buffer))?;

    stream.seek(SeekFrom::Start(attr_address))?;
    stream.write_all(&empty_block_srl)?;

    if header.first_empty > attr_address {
        new_first_empty()?;
    }

    Ok(())
}

// traverse linked list of attributes and delete along the tree
pub fn delete_attributes(attr_head: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    stream.seek(SeekFrom::Start(attr_head))?;

    let mut attr_address = attr_head;

    while attr_address > 0 {
        // read attr information
        let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<AttributeBlock>());
        stream.read_to_end(&mut buffer)?;
        let attr_block = map_bincode_error!(deserialize::<AttributeBlock>(&buffer))?;

        delete_attribute(attr_block.attribute)?;

        attr_address = attr_block.attribute.attr_next
    }

    Ok(())
}
