fn working_example() -> Result<()> {
    let info_test = InfoTest {
            id: 1,
            desc: "Description or name here...".to_string(),
            size: 2,
            offset: 3,
        };

    // Serialize the struct to a Vec<u8>
    let serialized = bincode::serialize(&info_test);

    // Write the serialized data to a binary file
    let mut file = File::create("serialized_data.bin")?;
    file.write_all(&serialized.unwrap())?;

    // Read the binary data from the file
    let mut file = File::open("serialized_data.bin")?;
    // let mut buffer = Vec::new();
    let struct_size = std::mem::size_of::<InfoTest>();
    let mut buffer: Vec<u8> = Vec::with_capacity(struct_size);
    file.read_exact(&mut buffer)?;

    // Deserialize the binary data back to a struct
    let x = bincode::deserialize(&buffer);
    if x.is_ok(){
        let deserialized: InfoTest = x.unwrap();
        println!("Deserialized: {:?}", deserialized);
        println!("ID: {}", deserialized.id);
        println!("desc: {}", deserialized.desc);
        println!("offset: {}", deserialized.offset);
    }
    Ok(())
}