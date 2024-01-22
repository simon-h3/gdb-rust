use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, Read, SeekFrom, Result, Error, ErrorKind};
use std::mem::size_of;
// use std::os::unix::fs::FileExt;
use bincode::{serialize, deserialize};

use crate::types::{Header, Node, Relationship, Attribute};                              // import structs
use crate::types::{Block, NodeBlock, RelationshipBlock, AttributeBlock, BlockType};     // import Block Types
use crate::types::PATH;                                                                // import db PATH

// custom error macro
macro_rules! custom_error {
    ($msg:expr) => {
        return Err(Error::new(ErrorKind::Other, $msg))
    };
}

// map bincode error to io error
macro_rules! map_bincode_error {
    ($expr:expr) => {
        $expr.map_err(|err| Error::new(ErrorKind::Other, format!("Bincode serialization error: {:?}", err)))
    };
}

// Format files used in DB - create header and empty blocks
pub fn format_disk(record_no: u64) -> Result<()>{
    let mut stream = File::create(PATH)?;
    let node_block_size = size_of::<NodeBlock>() as u64;
    let db_size: u64 = size_of::<Header>() as u64 + (node_block_size * record_no);

    let block: NodeBlock = Default::default();

    let header = Header {
        total_blocks: record_no.try_into().unwrap(),                // TODO: implement correctly (remove unwrap),
        first_empty: size_of::<Header>().try_into().unwrap(),  // or create a DEFAULT...
        db_size: db_size.try_into().unwrap(),
    };

    println!("Header: {:?}\r", header);

    let serialized_header = map_bincode_error!(serialize(&header))?;
    let serialized_block = map_bincode_error!(serialize(&block))?;

    // write header to file:
    stream.write_all(&serialized_header)?; // safe to unwrap as checked above

    // seek to first empty'
    stream.seek(SeekFrom::Start(header.first_empty))?;
    let mut offset = header.first_empty;

    assert_eq!(header.first_empty, size_of::<Header>() as u64);

    for _ in 0..header.total_blocks{
        stream.seek(SeekFrom::Start(offset))?;
        stream.write_all(&serialized_block)?;
        offset += node_block_size;
    }

    Ok(())
}

//  Grow output file when total blocks > blocks available, implemented to dynamically scale Database files.
// fn bool expandFile(const char* outfile, int newRecordNo);
fn expand_file(amount: u64) -> Result<()>{
    // open file in append mode:
    let mut stream = OpenOptions::new().append(true).open(PATH)?;

    // get current file size:
    let _current_size = stream.metadata()?.len();    // needed?

    // serialise Unset block
    let block: NodeBlock = Default::default(); // NodeBlock used but is set to Unset...
    let serialized_block = map_bincode_error!(serialize(&block))?;

    // write empty blocks to file:
    for _ in 0..amount {
        stream.write_all(&serialized_block)?;
    }

    // update header to reflect new db size
    // read header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let mut header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    // update header
    header.db_size += amount as usize * size_of::<NodeBlock>();
    header.total_blocks += amount;

    // write header
    stream.seek(SeekFrom::Start(0))?;
    let serialized_header = map_bincode_error!(serialize(&header))?;
    stream.write_all(&serialized_header)?;

    Ok(())
}

// Print header of file, given file name.
pub fn print_header() -> Result<()>{
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // read and print header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    println!("Header: {:?}\r", header);
    Ok(())
}

//  Given an offset print node to console.
pub fn print_node_name(offset: u64) -> Result<()>{
    let mut stream = OpenOptions::new().read(true).open(PATH).unwrap();

    // Move to offset
    stream.seek(SeekFrom::Start(offset))?;

    // Read bytes into Block struct
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let result_node = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;
    println!("-> {}", result_node.node.name);
    Ok(())
}

pub fn print_block(block: Block, buffer: Vec<u8>) -> Result<()>{
    match block.block_type {
        BlockType::Node => {
            let node_block = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;
            println!("Node: {:?}\r", node_block);
        }
        BlockType::Relationship => {
            let relationship_block = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;
            println!("Relationship: {:?}\r", relationship_block);
        }
        BlockType::Attribute => {
            let attribute_block = map_bincode_error!(deserialize::<AttributeBlock>(&buffer))?;
            println!("Attribute: {:?}\r", attribute_block);
        }
        BlockType::Empty => {
            println!("Empty found");
        }
        BlockType::Unset => {
            println!("Unset");
        }
    }

    Ok(())
}

//  Print any generic block given offset.
pub fn print_block_offset(offset: u64) -> Result<()>{
    let mut stream = OpenOptions::new().read(true).open(PATH)?;
    // Move to offset
    println!("Seeking -> Offset: {}\r", offset);
    stream.seek(SeekFrom::Start(offset))?;

    // Read bytes into Block struct
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let block = map_bincode_error!(deserialize::<Block>(&buffer))?;

    print_block(block, buffer)?;
    Ok(())
}

//  Print all blocks in file.
pub fn print_all_blocks() -> Result<()>{
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // read header
    let mut header_buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut header_buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&header_buffer))?;

    for i in 0..header.total_blocks {
        let curr_offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);

        // Move to offset
        stream.seek(SeekFrom::Start(curr_offset))?;
        let mut buffer = Vec::with_capacity(size_of::<NodeBlock>());
        stream.read_to_end(&mut buffer)?;

        let block = get_block(curr_offset)?;

        print_block(block, buffer)?;
    }

    Ok(())
}

fn print_relationship(relationship: &Relationship){
    println!("Relationship: {:?}\r", relationship);
}

fn get_first_empty(mut stream: &File, header: &Header) -> Result<u64> {
    const STRUCT_SIZE: u64 = size_of::<NodeBlock>() as u64;
    let mut curr_offset = size_of::<Header>() as u64;

    stream.seek(SeekFrom::Start(curr_offset))?; // move to first block

    for _ in 0..header.total_blocks {
        // Read bytes into Block struct
        let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
        stream.read_to_end(&mut buffer)?;   // TODO: find alternative to read_to_end...

        // let mut buffer: [u8; STRUCT_SIZE as usize] //= !needs initialising...;
        // stream.read_exact(&mut buffer)?;

        // Decode bytes into Block struct
        let block = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;

        // move to next block (for next iteration)
        curr_offset += STRUCT_SIZE;
        stream.seek(SeekFrom::Start(curr_offset))?;

        // return if block is empty or unset
        if block.block_type == BlockType::Empty || block.block_type == BlockType::Unset {
            return Ok(curr_offset - STRUCT_SIZE);
        }
    }
    // block not found, preventative option to expand?
    custom_error!("Error in getting first empty");
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

pub fn get_relationship(offset: u64) -> Result<Relationship>{
    let mut stream = File::open(PATH)?;
    stream.seek(SeekFrom::Start(offset))?;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());
    stream.read_to_end(&mut buffer)?;
    let relationship_block = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;

    return Ok(relationship_block.relationship);
}

pub fn get_block(offset: u64) -> Result<Block>{
    let mut stream = File::open(PATH)?;

    // Rewind the stream to the beginning
    stream.seek(SeekFrom::Start(offset))?;

    // Read the block from the stream
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let deserialised_block = map_bincode_error!(deserialize::<Block>(&buffer))?;

    // Return the block
    return Ok(deserialised_block);
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
    if new_first_empty == 0{
        expand_file(10)?;
        // create_node(&new_node);  //TODO: Remove possibility for 0 offset, expand automatically inside new_first_empty
        custom_error!("No first empty found, expanded file.")
    }
    else{
        println!("New First Empty: {}\r", new_first_empty);
        header.first_empty = new_first_empty;

        let serialized_header = map_bincode_error!(serialize(&header))?;
        stream.seek(SeekFrom::Start(0))?;
        stream.write_all(&serialized_header)?;

        println!(" - Create Node successful...\r\n");
        Ok(())
    }
}

pub fn create_relationship(new_relationship: Relationship) -> Result<()>{
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
        pad: [0; 16],
    };

    // write relationship information
    let serialized_relationship_block = map_bincode_error!(serialize(&relationship_block))?;
    stream.write_all(&serialized_relationship_block)?;

    // update first empty
    let new_first_empty = get_first_empty(&stream, &header)?;

    // update header
    if new_first_empty == 0{
        expand_file(10)?;
        // create_node(&new_node);  //TODO: recursive call back once expanded...??
        custom_error!("No first empty found, expanded file.")
    }
    else{
        let node = get_node_from_id(relationship_block.relationship.node_from)?;

        update_node_rlt(node, header.first_empty)?;

        // println!("New First Empty: {}\r", new_first_empty);
        header.first_empty = new_first_empty;

        let serialized_header = map_bincode_error!(serialize(&header))?;
        stream.seek(SeekFrom::Start(0))?;
        // stream.write_at(&serialized_header, 0)?; // linux only command (FileExt)
        stream.write_all(&serialized_header)?;

        println!(" - Create Relationship successful...\r\n");
    }

    Ok(())
}

pub fn create_attribute() -> Result<()>{
    let stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // TODO: ...
    

    Ok(())
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
pub fn get_node_address(node: &Node) -> Result<u64>{
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
//  Returns relationships address given a relationship
pub fn get_relationship_address(relationship: &Relationship) -> Result<u64>{
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // read header
    let mut buffer = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header.total_blocks{
        let offset = size_of::<Header>() as u64 + (i * size_of::<RelationshipBlock>() as u64);
        let current_relationship = get_relationship(offset)?;

        if compare_relationship(&current_relationship, &relationship){
            return Ok(offset);
        }
    }

    custom_error!("No Relationship Found, FATAL...");

}
//  Returns attributes address given an attribute
// Relationship getRelationshipToFrom(char* nameFrom, char* nameTo);

//  Returns attributes address given an attributes content

// fn u64 getAttributeAddressContent(char* content);

//  Returns attributes address given an attribute

// fn u64 getAttributeAddress(Attribute attribute);

//  Traverse file and print each block
pub fn print_all_nodes() -> Result<()>{
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
//  Print all relations FROM a node.
pub fn print_from_relations(node: &Node) -> Result<()>{
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    match node.rlt_head {
        0 => {
            println!("No relations found");
            return Ok(())
        }
        _ => {
            stream.seek(SeekFrom::Start(node.rlt_head))?;
            let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());
            stream.read_to_end(&mut buffer)?;
            let rlt = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;
            stream.seek(SeekFrom::Start(rlt.relationship.rlt_next))?;

            print_relationship(&rlt.relationship);

            while rlt.relationship.rlt_next != 0{
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
// fn printToRelations(fn u64 nodeOffset);

//  Print all attributes of a node.
// fn printAttributes(fn u64 nodeOffset);

//  If the relationships exists, extract data and write to file
// fn bool writeRelationship(const char* filename, Relationship relationship);

//  Create Attribute and write it to disk
// fn bool createAttribute(const char* filename, char* attrib);

// fn bool updateNodeName(fn u64 node, char* newNodeName);

fn append_relationship(node_address: u64, rlt_offset: u64) -> Result<()>{
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    stream.seek(SeekFrom::Start(node_address))?;

    let buffer: Vec<u8> = Vec::with_capacity(size_of::<RelationshipBlock>());
    let mut relationship_block = map_bincode_error!(deserialize::<RelationshipBlock>(&buffer))?;

    if relationship_block.relationship.rlt_next == 0{
        relationship_block.relationship.rlt_next = rlt_offset;
        stream.seek(SeekFrom::Start(node_address))?;

        let serialized_relationship_block = map_bincode_error!(serialize(&relationship_block))?;
        stream.write_all(&serialized_relationship_block)?;
        return Ok(())
    }
    else{
        append_relationship(relationship_block.relationship.rlt_next, rlt_offset)?;
    }

    Ok(())
}

fn append_attribute(node_address: u64, attribute_offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    stream.seek(SeekFrom::Start(node_address))?;

    let buffer: Vec<u8> = Vec::with_capacity(size_of::<AttributeBlock>());
    let mut attribute_block = map_bincode_error!(deserialize::<AttributeBlock>(&buffer))?;

    if attribute_block.attribute.attr_next == 0{
        attribute_block.attribute.attr_next = attribute_offset;
        stream.seek(SeekFrom::Start(node_address))?;

        let serialized_attribute_block = map_bincode_error!(serialize(&attribute_block))?;
        stream.write_all(&serialized_attribute_block)?;
        return Ok(())
    }
    else{
        append_attribute(attribute_block.attribute.attr_next, attribute_offset)?;
    }

    Ok(())
}

//  Retrospectively update nodes relationship list head upon creation, if already set follow and set to tail of list.
fn update_node_rlt(mut node: Node, rlt_offset: u64) -> Result<()>{
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // send a borrowed instance
    let node_address = get_node_address(&node)?;

    stream.seek(SeekFrom::Start(node_address))?;

    if node.rlt_head == 0{
        node.rlt_head = rlt_offset;

        let serialized_node = map_bincode_error!(serialize(&node))?;
        stream.seek(SeekFrom::Start(node_address))?;
        // stream.write_all_at(&serialized_node, node_address)?;    // linux only command (FileExt)
        stream.write_all(&serialized_node)?;

    }
    else {
        append_relationship(node_address, rlt_offset)?;
    }

    Ok(())
}
//  Retrospectively update nodes attribute list head upon creation, if already set follow and set to tail of list.
fn update_node_attribute(mut node: Node, attrib_offset: u64) -> Result<()>{
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let node_address = get_node_address(&node)?;


    if node.attr_head == 0{
        node.attr_head = attrib_offset;

        let serialized_node = map_bincode_error!(serialize(&node))?;
        stream.seek(SeekFrom::Start(node_address))?;
        // stream.write_all_at(&serialized_node, node_address)?;    // linux only command (FileExt)

        stream.seek(SeekFrom::Start(node_address))?;
        stream.write_all(&serialized_node)?;

    }
    else {
        append_attribute(node_address, attrib_offset)?;
    }

    Ok(())
}


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

