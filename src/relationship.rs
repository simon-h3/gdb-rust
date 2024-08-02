/*
    Simon H - 2024
*/

// fn bool deleteRelationshipRecouple(Relationship relationship, fn u64 nodeRltOffset);
use crate::disk::*;
use crate::node::*;

use bincode::{deserialize, serialize};
use libc::RTNLGRP_MCTP_IFADDR;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::mem::size_of;

// type imports can be combined, but this is easier to read
use crate::types;
use crate::types::{Block, BlockType, NodeBlock, RelationshipBlock}; // import Block Types
use crate::types::{Header, Node, Relationship}; // import structs
use crate::types::{PATH, RLT_PAD}; // import db PATH // import fixed static strings helper functions

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

pub fn print_relationship(relationship: &Relationship) {
    println!("Relationship: {:?}\r", relationship);
}

pub fn compare_relationship(rlt1: &Relationship, rlt2: &Relationship) -> bool {
    if rlt1.node_from == rlt2.node_from && rlt1.node_to == rlt2.node_to {
        return true;
    }
    false
}

pub fn get_relationship(offset: u64) -> Result<Relationship> {
    let mut stream = File::open(PATH)?;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());

    stream.seek(SeekFrom::Start(offset))?;
    stream.read_to_end(&mut buffer)?;

    let relationship_block = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;

    return Ok(relationship_block.relationship);
}

pub fn create_relationship(new_relationship: Relationship) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // read header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let mut header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    // go to first empty
    stream.seek(SeekFrom::Start(header.first_empty))?;

    let relationship_block = RelationshipBlock {
        block_type: BlockType::Relationship,
        relationship: new_relationship,
        pad: [0; RLT_PAD], // pad for consistent sizing across block types
    };

    // write relationship information
    let serialized_relationship_block = map_bincode_error!(serialize(&relationship_block))?;
    stream.write(&serialized_relationship_block)?;

    // update first empty
    let new_first_empty = get_first_empty(&stream, &header)?;

    // update header
    if new_first_empty == 0 {
        expand_file(10)?;
        create_relationship(new_relationship)?; //TODO: recursive call back once expanded...??
                                                // custom_error!("No first empty found, expanded file.")
    } else {
        let node = get_node_from_id(relationship_block.relationship.node_from)?;

        // update_node_rlt(node, header.first_empty)?;

        // println!("New First Empty: {}\r", new_first_empty);
        header.first_empty = new_first_empty;

        // write updated header
        let serialized_header = map_bincode_error!(serialize(&header))?;
        stream.seek(SeekFrom::Start(0))?;
        stream.write_all(&serialized_header)?;

        println!(" - Create Relationship successful...\r\n");
    }

    Ok(())
}

//  Returns relationships address given a relationship
pub fn get_relationship_address(relationship: &Relationship) -> Result<u64> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // read header
    let mut buffer = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<RelationshipBlock>() as u64);
        let current_relationship = get_relationship(offset)?;

        if compare_relationship(&current_relationship, &relationship) {
            return Ok(offset);
        }
    }

    custom_error!("No Relationship Found, FATAL...");
}

//  Returns attributes address given an attribute
pub fn get_relationship_from_to(name_from: &String, name_to: &String) -> Result<Relationship> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // let name_from = str_to_fixed_chars(&name_from);
    // let name_to = str_to_fixed_chars(&name_to);

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);
        let block = get_block(offset)?;

        match block.block_type {
            // need to specify full path (types::BlockType::)...
            types::BlockType::Relationship => {
                let node_from_address = get_node_address_from_name(&name_from)?;
                let node_to_address = get_node_address_from_name(&name_to)?;

                let relationship = get_relationship(offset)?;

                // TODO: switch to .eq??
                if relationship.node_from == node_from_address
                    && relationship.node_to == node_to_address
                {
                    return Ok(relationship); // yay :)
                }
            }
            // nah...
            _ => {}
        }
    }

    println!("Requested Relationship Non Existent..."); // TODO: real error needed...
    custom_error!("No Relationship Found, FATAL...");
}

//  Print all relations FROM a node.
pub fn print_from_relations(node: &Node) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    match node.rlt_head {
        0 => {
            println!("No relations found");
            return Ok(());
        }
        _ => {
            stream.seek(SeekFrom::Start(node.rlt_head))?;
            let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());
            stream.read_to_end(&mut buffer)?;
            let rlt = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;
            stream.seek(SeekFrom::Start(rlt.relationship.rlt_next))?;

            print_relationship(&rlt.relationship);

            while rlt.relationship.rlt_next != 0 {
                let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());
                stream.read_to_end(&mut buffer)?;
                let rlt = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;

                print_relationship(&rlt.relationship);

                stream.seek(SeekFrom::Start(rlt.relationship.rlt_next))?;
            }
        }
    }

    Ok(())
}

//  Print all relations TO a node.
pub fn print_to_relations(node_offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let mut header = map_bincode_error!(deserialize::<Header>(&buffer))?;
    stream.seek(SeekFrom::Start(node_offset))?;

    buffer = Vec::with_capacity(size_of::<NodeBlock>());

    let node_block = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;

    stream.seek(SeekFrom::Start(0))?;

    buffer = Vec::with_capacity(size_of::<RelationshipBlock>());

    for i in 0..header.total_blocks {
        stream.read_to_end(&mut buffer)?;
        let temp_rlt = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;

        if temp_rlt.relationship.node_to == node_block.node.id {
            print_block(map_bincode_error!(deserialize::<Block>(&buffer))?, &buffer)?;
        }
    }

    Ok(())
}

pub fn append_relationship(node_address: u64, rlt_offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let node = get_node(node_address)?;

    stream.seek(SeekFrom::Start(node.rlt_head))?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());

    stream.read_to_end(&mut buffer)?;

    let rlt = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;

    if rlt.relationship.rlt_next == 0 {
        rlt.relationship.rlt_next == rlt_offset;

        // TODO: fix
    } else {
        append_relationship(rlt.relationship.rlt_next, rlt_offset)?;
    }

    Ok(())
}

//  Assigns relationshipBlock to EMPTY_BLOCK and writes to disk
pub fn delete_relationship(relationship: Relationship) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let empty_block: NodeBlock = Default::default();
    let empty_block_srl = map_bincode_error!(serialize(&empty_block))?; // serialise

    let rlt_address = get_relationship_address(&relationship)?;

    // read current header information
    let mut header_buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut header_buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&header_buffer))?;

    stream.seek(SeekFrom::Start(rlt_address))?;

    // // read rlt_block
    // let mut rlt_buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());
    // stream.read_to_end(&mut rlt_buffer)?;
    // let rlt_block = map_bincode_error!(deserialize::<RelationshipBlock>(&rlt_buffer))?;

    stream.seek(SeekFrom::Start(rlt_address))?;

    stream.write_all(&empty_block_srl)?;

    if header.first_empty > rlt_address {
        new_first_empty()?;
    }

    Ok(())
}

// traverse linked list of relations and delete along the tree...
/*

    Open

    Seek to rlt_head

    while rlt_address != 0
        read

        delete
            set rlt_address to next

*/
pub fn delete_relations(rlt_head: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    stream.seek(SeekFrom::Start(rlt_head))?;

    let mut rlt_address = rlt_head;

    while rlt_address > 0 {
        // read rlt information
        let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());
        stream.read_to_end(&mut buffer)?;
        let rlt_block = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;

        delete_relationship(rlt_block.relationship)?;

        rlt_address = rlt_block.relationship.rlt_next
    }

    Ok(())
}
