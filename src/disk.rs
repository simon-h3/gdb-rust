use std::fs::{File, OpenOptions};
use std::io::{self, Write, Seek, Read, SeekFrom, Result, Error};
use std::mem::size_of;
use std::os::unix::fs::FileExt;
use bincode::{serialize, deserialize};

use crate::types::{Header, Node, Relationship, Attribute};                              // import structs
use crate::types::{Block, NodeBlock, RelationshipBlock, AttributeBlock, BlockType};     // import Block Types
use crate::types::PATH;                                                                // import db PATH

// custom error macro
macro_rules! custom_error {
    ($msg:expr) => {
        return Err(io::Error::new(io::ErrorKind::Other, $msg))
    };
}

// map bincode error to io error
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

    // TODO: Check if serialize macro better?
    let serialized_header = serialize(&header);
    let mut serialized_block = serialize(&block);

    if serialized_header.is_ok() && serialized_block.is_ok(){

        // write header to file:
        stream.write_all(&serialized_header.unwrap())?; // safe to unwrap as checked above

        // seek to first empty'
        stream.seek(SeekFrom::Start(header.first_empty))?;

        let mut offset = header.first_empty;
        // write empty blocks to file:
        // loop in increments of block size
        for _ in 0..record_no {
            serialized_block = serialize(&block);
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
fn expand_file(amount: u64) -> Result<()>{
    // open file in append mode:
    let mut stream = OpenOptions::new().append(true).open(PATH)?;

    // get current file size:
    let current_size = stream.metadata()?.len();    // needed? or can use seek?

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
    // let mut stream = File::open(PATH)?;   // should be open(PATH)...
    let mut stream = OpenOptions::new().read(true).open(PATH)?;
    // read and print header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    // let mut buffer: Vec<u8> = vec![0; 24];
    stream.read_to_end(&mut buffer)?;

    // TODO: remove
    // match read_result {
    //     Ok(_) => {
    //         println!("Read Result: {:?}\r", read_result);
    //     }
    //     Err(e) => {
    //         println!("Error in reading header: {:?}", e);
    //         return custom_error!("Error in reading header");
    //     }
    // }

    let result = deserialize::<Header>(&buffer);

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
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Block>());
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
        let mut buffer = Vec::with_capacity(size_of::<Block>());
        stream.read_to_end(&mut buffer)?;

        let block = get_block(curr_offset)?;

        // use block printing function instead...
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
            _ => {
                println!("Block Type Unknown...");
            }
        }
    }

    // println!("Returning aaaaa");
    Ok(())
}

fn get_first_empty(mut stream: &File, header: &Header) -> Result<u64> {
    // let mut buffer: Vec<u8> = vec![0; size_of::<Header>()];

    // stream.read_to_end(&mut buffer)?; // read header
    // let result = deserialize::<Header>(&buffer); // decode header

    let struct_size = size_of::<NodeBlock>() as u64;
    let mut curr_offset = size_of::<Header>() as u64;

    stream.seek(SeekFrom::Start(curr_offset)).unwrap(); // move to first block

    for _ in 0..header.total_blocks {
        // Read bytes into Block struct
        let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
        stream.read_to_end(&mut buffer)?;   // TODO: find alternative to read_to_end...

        // Decode bytes into Block struct
        let result = deserialize::<NodeBlock>(&buffer);

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
    let mut stream = File::open(PATH)?;

    // Rewind the stream to the beginning
    stream.seek(SeekFrom::Start(offset))?;

    // Read the block from the stream
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let deserialised_block = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;

    // Return the block
    return Ok(deserialised_block.node);
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
    let mut header;
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let header_result = deserialize::<Header>(&buffer);

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
    let serialized_node_block = serialize(&node_block);

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

                let serialized_header = serialize(&header);

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

    let header_result = deserialize::<Header>(&buffer);

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
    let serialized_relationship_block = serialize(&relationship_block);

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

            // let serialized_header = serialize(&header)?;
            let serialized_header = map_bincode_error!(serialize(&header))?;

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

    let header_result = map_bincode_error!(deserialize::<Header>(&buffer))?;

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

//  Basic Find node function TODO: test...
pub fn get_node_address(node: Node) -> Result<u64>{
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
    return Err(custom_error!("Not found, FATAL..."));
}
//  Returns relationships address given a relationship
// fn u64 getRelationshipAddress(Relationship relationship);

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
