// Module: test
#[cfg(test)]
mod tests {
    use crate::disk;
    use crate::types::{Node, Relationship};

    // default test to test if tests are working :)
    // #[test]
    // fn test_it_works() {
    //     let result = 2 + 2;
    //     assert_eq!(result, 4);
    // }

    #[test]
    fn format_test() {
        let result = disk::format_disk(10);
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
        let result = disk::format_disk(10);
        assert!(result.is_ok());
        disk::test_nodes();
        disk::test_relationships();

        // TEST
        let result = disk::print_all_blocks();
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_header() {
        // SETUP
        let result = disk::format_disk(10);
        assert!(result.is_ok());

        // TEST
        let result = disk::print_header();
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_creation() {
        // SETUP
        let result = disk::format_disk(10);
        assert!(result.is_ok());

        let test_node = Node {
            id: 0,
            name: "test".to_string(),
            rlt_head: 0,
            attr_head: 0,
        };

        // TEST
        let result = disk::create_node(test_node);
        assert!(result.is_ok());
    }

    #[test]
    fn test_relationship_creation(){
        // SETUP
        let result = disk::format_disk(10);
        assert!(result.is_ok());

        let test_relationship = Relationship {
            node_from: 0,
            node_to: 1,
            rlt_next: 0,
            attr_head: 0,
        };

        // TEST
        let result = disk::create_relationship(test_relationship);
        assert!(result.is_ok());
    }
}