/*
    Simon H - 2024
*/

mod types;  // Import the types module
mod disk;
mod test;
mod interface;
mod fixed_static_str;
mod api;

const TITLE: &str = r#"
            ___  ____   __   ____  _  _    ____   __  ____  __   ____   __   ____  ____ 
           / __)(  _ \ / _\ (  _ \/ )( \  (    \ / _\(_  _)/ _\ (  _ \ / _\ / ___)(  __)
          ( (_ \ )   //    \ ) __/) __ (   ) D (/    \ )( /    \ ) _ (/    \\___ \ ) _) 
           \___/(__\_)\_/\_/(__)  \_)(_/  (____/\_/\_/(__)\_/\_/(____/\_/\_/(____/(____)
        "#;

fn db_test() {
    types::assert_struct_size_equality();
    types::print_struct_info();

    println!("Format: {:?}", disk::format_disk(20));
    println!("Header: {:?}", disk::print_header());

    // println!("Block 1: {:?}", disk::print_block(24));

    // disk::print_first_empty();

    println!("Nodes: {:?}", test::test_nodes());

    // println!("Block 2: {:?}", disk::print_block(24));

    // disk::print_first_empty();

    println!("Relationships: {:?}\n", test::test_relationships());
    // let n = disk::print_block(24);

    println!("blocks: {:?}\n", disk::print_all_blocks());
    // disk::print_n_blocks(20).expect("Big no no in print_n_blocks...");

    println!("Header 2: {:?}\n", disk::print_header());

    println!("Export {:?}\n", disk::export_database());
}
fn main() {
    db_test();
    // interface::terminal_test();
}
