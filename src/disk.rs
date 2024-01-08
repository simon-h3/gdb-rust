use std::fs::{File, OpenOptions};
use std::io::{self, Write, Seek, Read, SeekFrom, Result};
use std::mem::size_of;
use std::os::unix::fs::FileExt;

use crate::types::{Block, Header, NodeBlock, Node, Relationship, Attribute, BlockType, RelationshipBlock};
use crate::types::PATH;

// custom error macro
macro_rules! custom_error {
    ($msg:expr) => {
        return Err(io::Error::new(io::ErrorKind::Other, $msg))
    };
}

macro_rules! map_bincode_error {
    ($expr:expr) => {
        $expr.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("Bincode serialization error: {:?}", err)))
    };
}

// Format files used in DB - create header and empty blocks
pub fn format_disk(record_no: usize) -> Result<()>{
    let mut stream = File::create(PATH)?;
    let node_block_size = size_of::<NodeBlock>();
    let db_size = size_of::<Header>() + (node_block_size * record_no);

    let block: NodeBlock = Default::default();

    let header = Header {
        total_blocks: record_no.try_into().unwrap(),                // TODO: implement correctly (remove unwrap),
        first_empty: size_of::<Header>().try_into().unwrap(),  // or create a DEFAULT...
        db_size: db_size.try_into().unwrap(),
    };

    println!("Header: {:?}\r", header);

    let serialized_header = bincode::serialize(&header);
    let mut serialized_block = bincode::serialize(&block);

    if serialized_header.is_ok() && serialized_block.is_ok(){

        // write header to file:
        stream.write_all(&serialized_header.unwrap())?; // safe to unwrap as checked above

        // seek to first empty'
        stream.seek(SeekFrom::Start(header.first_empty))?;

        let mut offset = header.first_empty;
        // write empty blocks to file:
        // loop in increments of block size
        for _ in 0..record_no {
            serialized_block = bincode::serialize(&block);
            stream.write_at(&serialized_block.unwrap(), offset)?; // safe to unwrap as checked above
            offset += node_block_size as u64;
        }

        println!(" - Format {} successful...\r", PATH);
        return Ok(());
    }

    Ok(())
}

//  Grow output file when total blocks > blocks available, implemented to dynamically scale Database files.
// fn bool expandFile(const char* outfile, int newRecordNo);

// Print header of file, given file name.
pub fn print_header() -> Result<()>{
    // let mut stream = File::open(PATH)?;   // should be open(PATH)...
    let mut stream = OpenOptions::new().read(true).open(PATH)?;
    // read and print header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    // let mut buffer: Vec<u8> = vec![0; 24];
    let read_result = stream.read_to_end(&mut buffer);

    match read_result {
        Ok(_) => {
            println!("Read Result: {:?}\r", read_result);
        }
        Err(e) => {
            println!("Error in reading header: {:?}", e);
            return custom_error!("Error in reading header");
        }
    }

    let result = bincode::deserialize::<Header>(&buffer);

    match result {
        Ok(header) => {
            println!("Header: {:?}\r", header);
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            return custom_error!("Error in Printing Header:");
        }
    }
}

//  Shortcut header printer (of all files)
// fn printHeaders();

//  Given an offset print node to console.
// fn printNodeName(fn offset: u64);

// Print attributes of a Relationship given a relationship
// fn printRelationship(relationship: Relationship);

//  Print any generic block given offset.
pub fn print_block(offset: u64) -> Result<()>{
    // let mut stream = File::create(PATH)?;   // should be open...
    let mut stream = OpenOptions::new().read(true).open(PATH)?;
    // Move to offset
    println!("Seeking -> Offset: {}\r", offset);
    stream.seek(SeekFrom::Start(offset))?;

    // Read bytes into Block struct
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    // let mut buffer: Vec<u8> = vec![0; 56];
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let result = bincode::deserialize::<NodeBlock>(&buffer);

    match result {
        Ok(block) => {
            println!("Block (Node): {:?}\r", block);
        }
        Err(e) => {
            println!("Error in printing block: {:?}", e);
        }
    }

    Ok(())
}

//  Print all blocks in file.
pub fn print_all_blocks() -> Result<()>{
    let mut stream = OpenOptions::new().read(true).open(PATH)?;
    let mut curr_offset = size_of::<Header>() as u64;

    // read header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let header_result = bincode::deserialize::<Header>(&buffer);

    match header_result {
        Ok(h) => {
            stream.seek(SeekFrom::Start(curr_offset))?;

            for _ in 0..h.total_blocks {
                // Read bytes into Block struct
                let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
                stream.read_to_end(&mut buffer)?;

                // Decode bytes into Block struct TODO: generic block, recast into corresponding block type
                let block = map_bincode_error!(bincode::deserialize::<Block>(&buffer))?;
                // move to next block (for next iteration)
                curr_offset += size_of::<NodeBlock>() as u64;
                stream.seek(SeekFrom::Start(curr_offset))?;

                match block.block_type {
                    BlockType::Empty => {
                        println!("Block (Empty): {:?}\r", block);
                    }
                    BlockType::Node => {
                        let node_block = map_bincode_error!(bincode::deserialize::<NodeBlock>(&buffer))?;
                        println!("Block (Node): {:?}\r", node_block);
                    }
                    BlockType::Relationship => {
                        let relationship_block = map_bincode_error!(bincode::deserialize::<RelationshipBlock>(&buffer))?;
                        println!("Block (Relationship): {:?}\r", relationship_block);
                    }
                    BlockType::Attribute => {
                        println!("Block (Attribute): {:?}\r", block);
                    }
                    BlockType::Unset => {
                        println!("Block (Unset): {:?}\r", block);
                    }
                }
            }
        }
        Err(e) => {
            // e incompatible with Result<()>...
            // return early
            println!("{}", e);
            return custom_error!("Error in reading header");
        }
    }

    Ok(())
}

fn get_first_empty(mut stream: &File, header: &Header) -> Result<u64> {
    // let mut buffer: Vec<u8> = vec![0; size_of::<Header>()];

    // stream.read_to_end(&mut buffer)?; // read header
    // let result = bincode::deserialize::<Header>(&buffer); // decode header

    let struct_size = size_of::<NodeBlock>() as u64;
    let mut curr_offset = size_of::<Header>() as u64;

    stream.seek(SeekFrom::Start(curr_offset)).unwrap(); // move to first block

    for _ in 0..header.total_blocks {
        // Read bytes into Block struct
        let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
        stream.read_to_end(&mut buffer)?;   // TODO: find alternative to read_to_end...

        // Decode bytes into Block struct
        let result = bincode::deserialize::<NodeBlock>(&buffer);

        // move to next block (for next iteration)
        curr_offset += struct_size;
        stream.seek(SeekFrom::Start(curr_offset))?;

        match result {
            Ok(block) => {
                if block.block_type == BlockType::Empty || block.block_type == BlockType::Unset {
                    return Ok(curr_offset - struct_size);
                }
            }
            Err(_e) => {
                // println!("Erroneous NodeBlock result: {:?}", e);
                // continue... could still return
            }
        }
    }
    return custom_error!("Error in getting first empty");   // header doesn't match file or header not read correctly
}

// Debug function
pub fn print_first_empty() -> Result<()> {
    let _stream = File::open(PATH)?;
    // println!("First Empty: {}", get_first_empty(&stream)?);
    Ok(())
}

//  fn boolean comparison between two Node structs
pub fn compare_node(node1: &Node, node2: &Node) -> bool {
    if node1.id == node2.id {
        return true;
    }
    false
}

//  fn boolean comparison between two Relationship structs
pub fn compare_relationship(rlt1: &Relationship, rlt2: &Relationship) -> bool {
    if rlt1.node_from == rlt2.node_from && rlt1.node_to == rlt2.node_to {
        return true;
    }
    false
}

pub fn compare_attribute(attrib1: &Attribute, attrib2: &Attribute) -> bool {
    if attrib1.value == attrib2.value {
        return true;
    }
    return false;
}

//  Given offset, return node structure
pub fn get_node(offset: u64) -> Result<Node> {
    let node: Node = Default::default();
    let mut stream = File::open(PATH)?;

    // Rewind the stream to the beginning
    stream.seek(SeekFrom::Start(offset))?;

    // Read the block from the stream
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let deserialised_block = map_bincode_error!(bincode::deserialize::<NodeBlock>(&buffer))?;

    // Return the block
    return Ok(deserialised_block.node);
}

//  Create Node and write it to disk
pub fn create_node(new_node: Node) -> Result<()> {
    // let mut stream = File::create(PATH)?;
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // read header
    let mut header;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let header_result = bincode::deserialize::<Header>(&buffer);

    match header_result {
        Ok(h) => {
            header = h;
        }
        Err(e) => {
            // e incompatible with Result<()>...
            // return early
            println!("{}", e);
            return custom_error!("Error in reading header");
        }
    }

    // go to first empty
    stream.seek(SeekFrom::Start(header.first_empty))?;

    let node_block = NodeBlock {
        block_type: BlockType::Node,
        node: new_node,
    };

    // write node information
    let serialized_node_block = bincode::serialize(&node_block);

    match serialized_node_block {
        Ok(bytes) => {
            stream.write_all(&bytes)?;
        }
        Err(e) => {
            // e incompatible with Result<()>...
            println!("{}", e);
            return custom_error!("Error in serializing node block");
        }
    }

    // update first empty
    let new_first_empty = get_first_empty(&stream, &header);

    // update header
    match new_first_empty {
        Ok(offset) => {
            if offset == 0{
                // TODO: handle erroneous or even full DB...
                return custom_error!("Error in getting new first empty");
            }
            else{
                println!("New First Empty: {}\r", offset);
                header.first_empty = offset;

                // write header
                stream.seek(SeekFrom::Start(0))?;

                let serialized_header = bincode::serialize(&header);

                match serialized_header {
                    Ok(bytes) => {
                        stream.write_all(&bytes)?;
                    }
                    Err(e) => {
                        // e incompatible with Result<()>...
                        println!("{}", e);
                        return custom_error!("Error in serializing header");
                    }
                }

                println!(" - Create Node successful...\r\n");
                Ok(())
            }
        }
        Err(e) => {
            println!("New First Empty Returned Error: {:?}", e);
            Err(e)
        }
    }
}

pub fn create_relationship(new_relationship: Relationship) -> Result<()>{
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // read header
    let mut header;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header_result = bincode::deserialize::<Header>(&buffer);

    match header_result {
        Ok(h) => {
            header = h;
        }
        Err(e) => {
            // e incompatible with Result<()>...
            // return early
            println!("{}", e);
            return custom_error!("Error in reading header");
        }
    }

    // go to first empty
    stream.seek(SeekFrom::Start(header.first_empty))?;

    let relationship_block = RelationshipBlock {
        block_type: BlockType::Relationship,
        relationship: new_relationship,
        pad: [0; 16],   // TODO: remove this...
    };

    // write relationship information
    let serialized_relationship_block = bincode::serialize(&relationship_block);

    match serialized_relationship_block {
        Ok(bytes) => {
            stream.write_all(&bytes)?;
        }
        Err(e) => {
            // e incompatible with Result<()>...
            println!("{}", e);
            return custom_error!("Error in serializing relationship block");
        }
    }

    // update first empty
    let new_first_empty = get_first_empty(&stream, &header);

    match new_first_empty{
        Ok(offset) => {
            header.first_empty = offset;

            // write header
            stream.seek(SeekFrom::Start(0))?;

            // let serialized_header = bincode::serialize(&header)?;
            let serialized_header = map_bincode_error!(bincode::serialize(&header))?;

            stream.write_all(&serialized_header)?;

            // // write if ok... TODO: investigate using ? on bincode...
            // if let Ok(bytes) = serialized_header {
            //     stream.write_all(&bytes)?;
            // } else if let Err(e) = serialized_header {
            //     println!("{}", e);
            //     return custom_error!("Error in serializing header");
            // }
        }
        Err(e) => {
            println!("New First Empty Returned Error: {:?}", e);
            return Err(e);
        }
    }

    Ok(())
}

//  Given id, return node Address
pub fn get_node_from_ID(id: u64) -> Result<Node> {
    // read header
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header_result = map_bincode_error!(bincode::deserialize::<Header>(&buffer))?;

    for i in 0..header_result.total_blocks {
        let offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);
        let node = get_node(offset)?;

        if node.id == id as usize {
            return Ok(node);
        }
    }

    return Err(custom_error!("Not found, FATAL..."));
}


//  Returns node address given node name
// fn u64 getNodeAddressFromName(char* nodeName);

//  Basic Find node function
// fn u64 getNodeAddress(Node* node);

//  Returns relationships address given a relationship
// fn u64 getRelationshipAddress(Relationship relationship);

//  Returns attributes address given an attribute
// Relationship getRelationshipToFrom(char* nameFrom, char* nameTo);

//  Returns attributes address given an attributes content

// fn u64 getAttributeAddressContent(char* content);

//  Returns attributes address given an attribute

// fn u64 getAttributeAddress(Attribute attribute);

//  Traverse file and print each block
// fn printAllNodes(const char* filename);

//  Print all relations FROM a node.
// fn printFromRelations(fn u64 nodeOffset);

//  Print all relations TO a node.
// fn printToRelations(fn u64 nodeOffset);

//  Print all attributes of a node.
// fn printAttributes(fn u64 nodeOffset);

//  If the relationships exists, extract data and write to file
// fn bool writeRelationship(const char* filename, Relationship relationship);

//  Create Attribute and write it to disk
// fn bool createAttribute(const char* filename, char* attrib);

// fn bool updateNodeName(fn u64 node, char* newNodeName);

//  Retrospectively update nodes relationship list head upon creation, if already set follow and set to tail of list.
// fn bool updateNodeRlt(fn u64 nodeAddress, fn u64 rltHead);

//  Retrospectively update nodes attribute list head upon creation, if already set follow and set to tail of list.
// fn bool updateNodeAttribute(fn u64 nodeAddress, fn u64 attribOffset);

//  Assigns relationshipBlock to EMPTY_BLOCK and writes to disk
// fn bool deleteRelationship(Relationship relationship);

//  Given a relationship remove its record
// fn bool deleteRelationshipRecouple(Relationship relationship, fn u64 nodeRltOffset);

//  Given an attribute remove its record
// fn bool deleteAttribute(Attribute attribute);

//  Given a nodes name remove its record
// fn bool deleteNodeName(char* name);

//  Given a Node remove its record
// fn bool deleteNode(Node node);

//  Given an offset and file, remove corresponding record
// fn bool deleteRecordOffset(const char* filename, fn u64 offset);

//  Export GDB for visualisation with Python
// fn bool exportGraphDatabase();

pub fn test_nodes() -> (){
    // define test nodes
    let node1 = Node {
        id: 1,
        name: "node1".to_string(),
        rlt_head: 0,
        attr_head: 0,
    };

    let node2 = Node {
        id: 2,
        name: "node2".to_string(),
        rlt_head: 0,
        attr_head: 0,
    };

    let node3 = Node {
        id: 3,
        name: "node3".to_string(),
        rlt_head: 0,
        attr_head: 0,
    };

    let _a = create_node(node1);
    let _b = create_node(node2);
    let _c = create_node(node3);

    // println!("1: {:?}", a);
    // println!("2: {:?}", b);
    // println!("3: {:?}", c);
}

pub fn test_relationships() -> (){
    let rlt1 = Relationship {
        node_from: get_node_from_ID(1).unwrap().id,
        node_to: get_node_from_ID(2).unwrap().id,
        rlt_next: 0,
        attr_head: 0,
    };

    let rlt2 = Relationship {
        node_from: get_node_from_ID(2).unwrap().id,
        node_to: get_node_from_ID(3).unwrap().id,
        rlt_next: 0,
        attr_head: 0,
    };

    let rlt3 = Relationship {
        node_from: get_node_from_ID(3).unwrap().id,
        node_to: get_node_from_ID(1).unwrap().id,
        rlt_next: 0,
        attr_head: 0,
    };

    let _a = create_relationship(rlt1);
    let _b = create_relationship(rlt2);
    let _c = create_relationship(rlt3);
}
