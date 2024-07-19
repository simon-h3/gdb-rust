# RUST - GRAPH DATABASE
~~- [ ] TUI (not ncurses) Menu System~~
- [ ] Populate From File
- [ ] Populate From HTTP Endpoint
- [x] Format Disk
- [x] Expand Disk
- [x] Print Headers
- [x] Node, Relationship, Attribute and Block Printers
- [x] Node, Relationship, Attribute comparisons.
- [x] Node, Relationship, Attribute 'getters'.
- [x] Create Node
- [x] Create Relationship
- [x] Create Attribute
- [ ] Delete Node
- [ ] Delete Relationship
- [ ] Delete Attribute

# PYTHON - GRAPH VISUALISATION
- [x] JSON Export
- [x] Visualisation with Python

# Database Logic

Upon running a fresh database, a default set of blocks are 'formatted' allowing for the filling of new node, relationship and attribute blocks.

There is always a reserved final block at the end for automatic protection on file operations and any buffers causing issues (EOF error etc).

# Self Notes

- Not safe from long string sizes, partly due to method of converting strings into char arrays.

- Sequential access will only get slower over time - potential for sorting, separating and shuffling blocks.
