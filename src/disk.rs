/*
    Simon H - 2024
*/

use bincode::{deserialize, serialize};
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::os::unix::fs::FileExt;

// type imports can be combined, but this is easier to read
use crate::types::Header; // import structs
use crate::types::{AttributeBlock, Block, BlockType, NodeBlock, RelationshipBlock}; // import Block Types
use crate::types::{EXPORT_PATH, PATH};

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

// Format files used in DB - create header and empty blocks
pub fn format_disk(record_no: u64) -> Result<()> {
    let mut stream = File::create(PATH)?;
    let node_block_size = size_of::<NodeBlock>() as u64;
    let db_size: u64 = size_of::<Header>() as u64 + (node_block_size * record_no) + 56 * 2; // TODO: make consistent... -> added padding, prevents EoF errors

    let block: NodeBlock = Default::default();

    let header = Header {
        total_blocks: record_no.try_into().unwrap(), // TODO: implement correctly (remove unwrap),
        first_empty: size_of::<Header>().try_into().unwrap(), // or create a DEFAULT...
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

    for _ in 0..header.total_blocks {
        // stream.seek(SeekFrom::Start(offset))?;
        stream.write_at(&serialized_block, offset)?;
        offset += node_block_size;
    }

    let mut final_block: Block = Default::default();
    final_block.block_type = BlockType::Final;

    let serialized_final_block = map_bincode_error!(serialize(&final_block))?;

    stream.write_at(&serialized_final_block, offset)?;

    Ok(())
}

// Grow output file when total blocks > blocks available, implemented to dynamically scale Database files.
pub fn expand_file(amount: u64) -> Result<()> {
    println!("Expanding file...");
    // open file in append mode:
    let mut stream = OpenOptions::new().append(true).open(PATH)?;

    // get current file size:
    let _current_size = stream.metadata()?.len(); // needed?

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
    header.db_size += amount * size_of::<NodeBlock>() as u64;
    header.total_blocks += amount;

    // write header
    stream.seek(SeekFrom::Start(0))?;
    let serialized_header = map_bincode_error!(serialize(&header))?;
    stream.write_all(&serialized_header)?;

    Ok(())
}

/* potentially redundant...
pub fn populate_from_file() -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open()?;

    Ok(())
}
*/

// Print header of file, given file name.
pub fn print_header() -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // read and print header
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;

    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    println!("Header: {:?}\r", header);
    Ok(())
}

//  Given an offset print node to console.
// pub fn print_node_name(offset: u64) -> Result<()> {
//     let mut stream = OpenOptions::new().read(true).open(PATH).unwrap();

//     // Move to offset
//     stream.seek(SeekFrom::Start(offset))?;

//     // Read bytes into Block struct
//     let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
//     stream.read_to_end(&mut buffer)?;

//     // Decode bytes into Block struct
//     let result_node = map_bincode_error!(deserialize::<NodeBlock>(&buffer))?;
//     println!("-> {}", str_conversion::char_print(&result_node.node.name));
//     Ok(())
// }

pub fn print_block(block: Block, buffer: &Vec<u8>) -> Result<()> {
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
            println!("Unknown...");
        }
    }

    Ok(())
}

//  Print any generic block given offset.
pub fn print_block_offset(offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;
    // Move to offset
    println!("Seeking -> Offset: {}\r", offset);
    stream.seek(SeekFrom::Start(offset))?;

    // Read bytes into Block struct
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let block = map_bincode_error!(deserialize::<Block>(&buffer))?;

    print_block(block, &buffer)?;
    Ok(())
}

//  Print all blocks in file.
pub fn print_all_blocks() -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    // read header
    let mut header_buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut header_buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&header_buffer))?;

    stream.seek(SeekFrom::Start(size_of::<Header>() as u64))?; // move to first block (after header)

    for i in 0..header.total_blocks {
        let curr_offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);
        // let curr_offset = 24 + (i * 56);
        // Move to offset
        stream.seek(SeekFrom::Start(curr_offset))?;
        let mut buffer = Vec::with_capacity(size_of::<NodeBlock>());
        stream.read_to_end(&mut buffer)?;

        let block = get_block(curr_offset)?;
        println!("@: {:?}\r", curr_offset);
        print_block(block, &buffer)?;
    }

    Ok(())
}

pub fn print_n_blocks(n: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).open(PATH)?;

    let mut header_buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_exact(&mut header_buffer)?;

    stream.seek(SeekFrom::Start(size_of::<Header>() as u64))?;

    for i in 0..n {
        let curr_offset = size_of::<Header>() as u64 + (i * size_of::<NodeBlock>() as u64);

        stream.seek(SeekFrom::Start(curr_offset))?;
        // let mut buffer = [0u8; 96];
        let mut buffer = Vec::with_capacity(96);
        println!("Buffer size at {} -> {}", i, std::mem::size_of_val(&buffer));
        stream.read_exact(&mut buffer)?;

        let block = get_block(curr_offset)?;
        println!("@: {:?}\r", curr_offset);
        print_block(block, &buffer)?;
    }

    Ok(())
}

pub fn get_first_empty(mut stream: &File, header: &Header) -> Result<u64> {
    const STRUCT_SIZE: u64 = size_of::<Block>() as u64;
    let mut curr_offset = size_of::<Header>() as u64;

    stream.seek(SeekFrom::Start(curr_offset))?; // move to first block

    // TODO: check logic
    for _ in 0..header.total_blocks {
        // Read bytes into Block struct
        let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Block>());
        stream.read_to_end(&mut buffer)?; // TODO: find alternative to read_to_end...

        // let mut buffer: [u8; STRUCT_SIZE as usize] //= !needs initialising...;
        // stream.read_to_end_exact(&mut buffer)?;

        // Decode bytes into Block struct
        let block = map_bincode_error!(deserialize::<Block>(&buffer))?;

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

// update first empty
pub fn new_first_empty() -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    // read current header information
    let mut header_buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut header_buffer)?;
    let mut header = map_bincode_error!(deserialize::<Header>(&header_buffer))?;

    let new_first_empty = get_first_empty(&stream, &header)?;

    header.first_empty = new_first_empty;

    // write new header information
    stream.seek(SeekFrom::Start(0))?;
    let serialized_header = map_bincode_error!(serialize(&header))?;
    stream.write_all(&serialized_header)?;

    Ok(())
}

// Debug function
pub fn print_first_empty() -> Result<()> {
    let _stream = File::open(PATH)?;
    // println!("First Empty: {}", get_first_empty(&stream)?);
    Ok(())
}

pub fn get_block(offset: u64) -> Result<Block> {
    let mut stream = File::open(PATH)?;

    // Rewind the stream to the beginning
    stream.seek(SeekFrom::Start(offset))?;

    // Read the block from the stream
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<NodeBlock>());
    stream.read_to_end(&mut buffer)?;

    // Decode bytes into Block struct
    let deserialised_block = deserialize::<Block>(&buffer);

    return match deserialised_block {
        Ok(block_ok) => {
            // println!("Block Okay!");
            Ok(block_ok)
        }
        Err(block_not_ok) => {
            println!("Block Not Okay!");
            // bad
            custom_error!(block_not_ok);
            // exit(1);
        }
    };

    // Return the block
    // return Ok(deserialised_block);
}

//  Given an offset and file, remove corresponding record
// TODO: test
pub fn delete_record_offset(offset: u64) -> Result<()> {
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    stream.seek(SeekFrom::Start(offset))?;

    // read block information
    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Block>());
    stream.read_to_end(&mut buffer)?;
    let mut block = map_bincode_error!(deserialize::<Block>(&buffer))?;

    block.block_type = BlockType::Empty;

    // write block information
    let serialized_block = map_bincode_error!(serialize(&block))?;
    stream.seek(SeekFrom::Start(offset))?;
    stream.write_all(&serialized_block)?;

    Ok(())
}

//  Export GDB for visualisation with Python
pub fn export_database() -> Result<()> {
    /*
       Serialise all nodes, relationships, attributes into JSON
       for ease later when parsing in visualisation tool...
    */

    let mut out_stream = OpenOptions::new()
        .create(true)
        .write(true)
        .open(EXPORT_PATH)?;
    let mut stream = OpenOptions::new().read(true).write(true).open(PATH)?;

    let mut buffer: Vec<u8> = Vec::with_capacity(size_of::<Header>());
    stream.read_to_end(&mut buffer)?;
    let header = map_bincode_error!(deserialize::<Header>(&buffer))?;

    for i in 0..header.total_blocks {
        // let mut block = get_block(i * size_of::<NodeBlock>() as u64)?;

        let mut block_buffer = Vec::with_capacity(size_of::<NodeBlock>());
        stream.seek(SeekFrom::Start(
            i * size_of::<NodeBlock>() as u64 + size_of::<Header>() as u64,
        ))?;
        stream.read_to_end(&mut block_buffer)?;
        let block = map_bincode_error!(deserialize::<Block>(&block_buffer))?;

        match block.block_type {
            BlockType::Node => {
                let struct_data: NodeBlock =
                    map_bincode_error!(deserialize::<NodeBlock>(&block_buffer))?;
                let json_string = serde_json::to_string(&struct_data.node)?;
                // writeln!(stream, "{}", json_string)?;
                out_stream
                    .write_all(json_string.as_bytes())
                    .expect("Failed to write to file");
                out_stream
                    .write_all(b"\n")
                    .expect("Failed to write to file"); // Add a newline after each JSON object
            }
            BlockType::Relationship => {
                let struct_data: RelationshipBlock =
                    map_bincode_error!(deserialize::<RelationshipBlock>(&block_buffer))?;
                let json_string = serde_json::to_string(&struct_data.relationship)?;
                // writeln!(stream, "{}", json_string)?;
                out_stream
                    .write_all(json_string.as_bytes())
                    .expect("Failed to write to file");
                out_stream
                    .write_all(b"\n")
                    .expect("Failed to write to file"); // Add a newline after each JSON object
            }
            BlockType::Attribute => {
                let struct_data: AttributeBlock =
                    map_bincode_error!(deserialize::<AttributeBlock>(&block_buffer))?;
                let json_string = serde_json::to_string(&struct_data.attribute)?;
                // writeln!(stream, "{}", json_string)?;
                out_stream
                    .write_all(json_string.as_bytes())
                    .expect("Failed to write to file");
                out_stream
                    .write_all(b"\n")
                    .expect("Failed to write to file"); // Add a newline after each JSON object
            }
            BlockType::Empty => {
                // do nothing
            }
            BlockType::Unset => {
                // do nothing
            }
            BlockType::Final => {
                // do nothing
            }
        }
    }

    Ok(())
}
