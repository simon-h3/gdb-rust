mod types;  // Import the types module
mod disk;
mod test;

const TITLE: &str = r#"
            ___  ____   __   ____  _  _    ____   __  ____  __   ____   __   ____  ____ 
           / __)(  _ \ / _\ (  _ \/ )( \  (    \ / _\(_  _)/ _\ (  _ \ / _\ / ___)(  __)
          ( (_ \ )   //    \ ) __/) __ (   ) D (/    \ )( /    \ ) _ (/    \\___ \ ) _) 
           \___/(__\_)\_/\_/(__)  \_)(_/  (____/\_/\_/(__)\_/\_/(____/\_/\_/(____/(____)
        "#;

fn db_test() {
    types::print_struct_info();

    println!("Format: {:?}", disk::format_disk(15));
    println!("Header: {:?}", disk::print_header());

    // println!("Block 1: {:?}", disk::print_block(24));

    // disk::print_first_empty();

    println!("Nodes: {:?}", test::test_nodes());

    // println!("Block 2: {:?}", disk::print_block(24));

    // disk::print_first_empty();

    println!("Relationships: {:?}", test::test_relationships());
    // let n = disk::print_block(24);

    println!("blocks: {:?}", disk::print_all_blocks());

    println!("Header 2: {:?}", disk::print_header());
}
fn main() {
    db_test();
}
