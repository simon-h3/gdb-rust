use std::fs::{File, OpenOptions};
use std::io::{self, Write, Seek, Read, SeekFrom, Result};
use std::mem;

use crate::types::{Header, NodeBlock, Node, Relationship, Attribute, BlockType};
use crate::types::PATH;

// Helper function to get a mutable reference to a slice
fn raw_bytes_mut<T>(data: &mut T) -> &mut [u8] {
    unsafe {
        std::slice::from_raw_parts_mut(data as *mut T as *mut u8, mem::size_of_val(data))
    }
}

// Format files used in DB - create header and empty blocks
pub fn format_disk(record_no: usize) -> Result<()>{
    let mut stream = File::create(PATH)?;
    let db_size = mem::size_of::<Header>() + (mem::size_of::<NodeBlock>() * record_no);

    let mut block: NodeBlock = Default::default();

    let mut header = Header {
        total_blocks: record_no.try_into().unwrap(),                // implement correctly (remove unwrap),
        first_empty: mem::size_of::<Header>().try_into().unwrap(),  // or create a DEFAULT...
        db_size: db_size.try_into().unwrap(),
    };

    println!("Header: {:?}\r", header);

    // write header to file:
    stream.write_all(raw_bytes_mut(&mut header))?;
    
    // seek to first empty'
    stream.seek(SeekFrom::Start(header.first_empty))?;

    for _ in 0..record_no {
        stream.write_all(raw_bytes_mut(&mut block))?;
        // stream.write_all(&serialized_block)?;
    }

    println!(" - Format {} successful...\r", PATH);
    Ok(())
}

//  Grow output file when total blocks > blocks available, implemented to dynamically scale Database files.
// fn bool expandFile(const char* outfile, int newRecordNo);

// Print header of file, given file name.
pub fn print_header() -> Result<()>{
    let mut stream = File::open(PATH)?;
    let mut header: Header = Default::default();

    // read and print header
    // let mut buffer: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Header>());
    let mut buffer: Vec<u8> = vec![0; 24];
    stream.read_to_end(&mut buffer)?;

    let result = bincode::deserialize(&buffer);

    if result.is_ok(){
        header = result.unwrap();
        println!("Header: {:?}\r", header);
    }
    else{
        println!("in reading header... Error: {:?}", result);
    }
    Ok(())
}

//  Shortcut header printer (of all files)
// fn printHeaders();

//  Given an offset print node to console.
// fn printNodeName(fn offset: u64);

// Print attributes of a Relationship given a relationship
// fn printRelationship(relationship: Relationship);

//  Print any generic block given offset.
pub fn print_block(offset: u64) -> Result<()>{
    let mut stream = File::open(PATH)?;
    let mut block = Default::default();

    // Move to offset
    println!("Seeking -> Offset: {}\r", offset);
    let n = stream.seek(SeekFrom::Start(offset));

    // Read bytes into Block struct
    // let mut buffer: Vec<u8> = Vec::with_capacity(std::mem::size_of::<NodeBlock>());
    let mut buffer: Vec<u8> = vec![0; 56];
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let result = bincode::deserialize::<NodeBlock>(&buffer);

    if result.is_ok(){
        block = result.unwrap();
        println!("Block Info: {:?}\r", block);
    }
    else{
        println!("in reading header... Error: {:?}", result);
    }

    Ok(())
}

//  Get header from file and return
/*pub fn get_first_empty() -> Result<u64> {
    let mut stream = File::open(PATH)?;
    let mut header: Header = Default::default();
    let mut block: NodeBlock = Default::default();

    stream.seek(SeekFrom::Start(0))?;

    // Read the header from the stream
    let header_size = std::mem::size_of::<Header>();
    let mut buffer: Vec<u8> = Vec::with_capacity(header_size.try_into().unwrap());
    stream.read_to_end(&mut buffer)?;
    let result = bincode::deserialize(&buffer);

    if result.is_ok(){
        header = result.unwrap();
        println!("Header: {:?}\r", header);
    }
    else{
        println!("in reading header... Error: {:?}", result);
    }

    let mut stream = File::open(PATH)?;

    let struct_size = mem::size_of::<NodeBlock>() as u64;
    let mut curr_offset = 0;

    for offset in 0..header.total_blocks{
        // Move to offset
        curr_offset = header_size as u64 + (offset * struct_size) as u64;
        println!("Seeking -> Offset: {}\r", curr_offset);
        let n = stream.seek(SeekFrom::Start(curr_offset as u64))?;
        println!("Seeked Amount -> Offset: {}\r", n);
        // Read bytes into Block struct
        let mut buffer2: Vec<u8> = Vec::with_capacity(struct_size.try_into().unwrap());
        stream.read_exact(&mut buffer2)?;

        // Decode bytes into Block struct
        let result2 = bincode::deserialize::<NodeBlock>(&buffer);
        println!("Result2: {:?}\r", result2);

        match result2 {
            Ok(decoded_block) => {
                block = decoded_block;
                println!("BlockNode: {:?}\r", block);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
            // block = result2.unwrap();
            // println!("BlockNode: {:?}\r", block);

            // if block.block_type == BlockType::Empty {
            //     break;
            // }
        }
    }
    Ok(curr_offset)
}*/

fn get_first_empty(mut stream: &File) -> u64 {
    let mut buffer: Vec<u8> = vec![0; mem::size_of::<Header>()];

    stream.read_to_end(&mut buffer).expect("TODO: panic message");  // read header
    let result = bincode::deserialize::<Header>(&buffer);           // decode header

    match result {
        Ok(header) => {
            let struct_size = mem::size_of::<NodeBlock>() as u64;
            let mut curr_offset = mem::size_of::<Header>() as u64;
            
            stream.seek(SeekFrom::Start(mem::size_of::<Header>() as u64)).unwrap(); // move to first block

            for _ in 0..header.total_blocks {
                // Read bytes into Block struct
                let mut buffer: Vec<u8> = vec![0; mem::size_of::<NodeBlock>()];
                stream.read_to_end(&mut buffer).expect("TODO: panic message");

                // Decode bytes into Block struct
                let result = bincode::deserialize::<NodeBlock>(&buffer);

                match result {
                    Ok(block) => {
                        println!("BlockNode: {:?}\r", block);

                        if block.block_type == BlockType::Empty {
                            return curr_offset - mem::size_of::<NodeBlock>() as u64;
                        }
                        curr_offset += struct_size;
                    }
                    Err(e) => {
                        println!("Erroneous NodeBlock result: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e); // TODO: panic message
        }
    }

    // if result.is_ok(){
    //     let header = result.unwrap();
    //     println!("Header: {:?}\r", header);

    //     for _ in 0..header.total_blocks {
    //         // Read bytes into Block struct
    //         let mut buffer: Vec<u8> = vec![0; mem::size_of::<NodeBlock>()];
    //         stream.read_to_end(&mut buffer).expect("TODO: panic message");

    //         // Decode bytes into Block struct
    //         let result = bincode::deserialize::<NodeBlock>(&buffer);

    //         if result.is_ok(){
    //             let block = result.unwrap();
    //             println!("BlockNode: {:?}\r", block);

    //             if block.block_type == BlockType::Empty {
    //                 return stream.seek(SeekFrom::Current(0)).unwrap() - mem::size_of::<NodeBlock>() as u64;
    //             }
    //         }
    //     }
    // }
    // else{
    //     println!("in reading header... Error: {:?}", result);
    // }
    return 0;
}

// Debug function
pub fn print_first_empty() -> Result<()> {
    let stream = File::open(PATH)?;
    println!("First Empty: {}", get_first_empty(&stream));
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

pub fn compareAttribute(attrib1: &Attribute, attrib2: &Attribute) -> bool {
    if attrib1.value == attrib2.value {
        return true;
    }
    return false;
}

//  Given offset, return node structure
pub fn get_node(offset: &usize) -> Result<Node> {
    let mut node: Node = Default::default();
    let mut stream = File::open(PATH)?;

    // Rewind the stream to the beginning
    stream.seek(SeekFrom::Start(*offset as u64))?;

    // Read the block from the stream
    let mut buffer: Vec<u8> = Vec::with_capacity(std::mem::size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;


    // let result = bincode::deserialize(&buffer);
    
    // if result.is_ok(){
    //     let block: NodeBlock = result.unwrap();
    //     node = block.node;
    //     return Ok(node)
    // }
    // else{
    //     println!("In get-node Error: {:?}", result);
    // }

    Ok(node)
}

//  Create Node and write it to disk
pub fn create_node(new_node: Node) -> Result<()> {
    // let mut stream = File::open(PATH)?;
    let mut stream = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(PATH)?;

    let mut header: Header = Default::default();

    // read header
    // let mut buffer: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Header>());
    let mut buffer: Vec<u8> = vec![0; 24];
    stream.read_to_end(&mut buffer).expect("TODO: panic message");

    // let mut stream = File::create(PATH)?;

    // go to first empty
    stream.seek(SeekFrom::Start(header.first_empty))?;

    let mut node_block = NodeBlock {
        block_type: BlockType::Node,
        node: new_node,
    };

    // write node information
    stream.write_all(raw_bytes_mut(&mut node_block))?;

    // update first empty
    let new_first_empty = get_first_empty(&stream);
    if new_first_empty == 0{
        // TODO: handle erroneous or even full DB...
        println!("Panic!");
        Ok(())
    }
    else{
        println!("New First Empty: {}\r", new_first_empty);
        header.first_empty = new_first_empty;

        // write header
        stream.seek(SeekFrom::Start(0)).expect("TODO: panic message");
        stream.write_all(raw_bytes_mut(&mut header))?;

        println!(" - Create Node successful...\r\n");
        Ok(())
    }
}

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

//  Given id, return node Address
// fn u64 getNodeFromID(int id);

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

//  Create Relationship and write to disk
// fn bool createRelationship(const char* filename, fn u64 nodeFrom, fn u64 nodeTo);

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
