from pyvis.network import Network
import hashlib
import json

INPUT_PATH = "database/output.json"
OUTPUT_PATH = "database/graph.html"

# Hash function from module name (purely aesthetics)
def letter_to_hex(str):
    """
    A function that takes a string input and converts it to a hexadecimal colour value.
    
    Parameters:
    str (str): The input string to be converted to a hexadecimal colour value.
    
    Returns:
    str: The hexadecimal color value generated from the input string.
    """
    sha256_hash = hashlib.sha256(str.encode()).hexdigest()

    # Take the first six characters of the hash and convert to RGB values
    r = int(sha256_hash[:2], 16)
    g = int(sha256_hash[2:4], 16)
    b = int(sha256_hash[4:6], 16)

    # Convert the RGB values to a hexadecimal color
    hex_color = "#{:02x}{:02x}{:02x}".format(r, g, b)

    return hex_color


def load_lines(path):
    with open(path) as f:
        lines = f.readlines()
    return lines


def generate_graph():
    """
    Generate a graph using the Network class, add nodes and edges from the loaded lines, and display the graph.
    """
    net = Network(directed=True, height=1200)
    net.repulsion()
    net.show_buttons(filter_=['physics'])
    lines = load_lines(INPUT_PATH)

    # Add nodes
    for line in lines:
        json_obj = json.loads(line)
        
        if "name" in line:
            net.add_node(json_obj['id'], label=json_obj['name'], color=letter_to_hex(json_obj['name']))

    # Add edges
    for line in lines:
        json_obj = json.loads(line)

        if "node_from" in line:
            net.add_edge(json_obj['node_from'], json_obj['node_to'])

    net.show(OUTPUT_PATH, notebook=False)


if __name__ == "__main__":
    generate_graph()
