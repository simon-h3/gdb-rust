use crate::disk::{create_node, create_relationship};
use crate::types::{Node, Relationship};

// Module: test
#[cfg(test)]
mod tests {
    use crate::disk::{create_node, create_relationship, format_disk, print_all_blocks, print_header};
    use crate::test::{test_nodes, test_relationships};
    use crate::types::{Node, Relationship};

    // default test to test if tests are working :)
    // #[test]
    // fn test_it_works() {
    //     let result = 2 + 2;
    //     assert_eq!(result, 4);
    // }

    #[test]
    fn format_test() {
        let result = format_disk(10);
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_relationships() {
    //     let result = disk::test_relationships();
    //     assert_eq!(result, ());
    // }

    #[test]
    fn test_print_all_blocks() {
        // SETUP
        let result_format = format_disk(10);
        assert!(result_format.is_ok());
        test_nodes();
        let rlt_result = test_relationships();
        assert!(rlt_result.is_ok());

        // TEST
        let result = print_all_blocks();
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_header() {
        // SETUP
        let result = format_disk(10);
        assert!(result.is_ok());

        // TEST
        let result = print_header();
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_creation() {
        // SETUP
        let result = format_disk(10);
        assert!(result.is_ok());

        let test_node = Node {
            id: 0,
            name: "test".to_string(),
            rlt_head: 0,
            attr_head: 0,
        };

        // TEST
        let result = create_node(test_node);
        assert!(result.is_ok());
    }

    #[test]
    fn test_relationship_creation(){
        // SETUP
        let result = format_disk(10);
        assert!(result.is_ok());

        let test_relationship = Relationship {
            node_from: 0,
            node_to: 1,
            rlt_next: 0,
            attr_head: 0,
        };

        // TEST
        let result = create_relationship(test_relationship);
        assert!(result.is_ok());
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

    let a = create_node(node1);
    let b = create_node(node2);
    let c = create_node(node3);

    // println!("1: {:?}", a);
    // println!("2: {:?}", b);
    // println!("3: {:?}", c);
}

pub fn test_relationships() -> std::io::Result<()> {
    let rlt1 = Relationship {
        node_from: 1,
        node_to: 2,
        rlt_next: 0,
        attr_head: 0,
    };

    let rlt2 = Relationship {
        node_from: 2,
        node_to: 3,
        rlt_next: 0,
        attr_head: 0,
    };

    let rlt3 = Relationship {
        node_from: 3,
        node_to: 1,
        rlt_next: 0,
        attr_head: 0,
    };

    println!("{:?}", rlt1);
    println!("{:?}", rlt2);
    println!("{:?}", rlt3);

    create_relationship(rlt1)?;
    create_relationship(rlt2)?;
    create_relationship(rlt3)?;

    println!("RltS creation successful...");

    Ok(())
}
