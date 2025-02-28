#!/usr/bin/env python
import networkx as nx
import sys

def read_graph_from_file(file_path):
    G = nx.Graph()
    with open(file_path, 'r') as file:
        for line in file:
            if line.startswith("#"):
                continue
            node1, node2 = map(int, line.split())
            G.add_edge(node1, node2)
    return G

def calculate_coreness(G):
    coreness = nx.core_number(G)
    return coreness

def save_coreness_to_file(coreness, output_file):
    try:
        with open(output_file, 'w') as file:
            for node, coreness_value in sorted(coreness.items()):
                file.write(f"{coreness_value}\n")
    except Exception as e:
        print(e)

def main(input_file, output_file):
    G = read_graph_from_file(input_file)
    
    coreness = calculate_coreness(G)
    
    save_coreness_to_file(coreness, output_file)
    print(f"Coreness salvata in {output_file}")

if __name__ == "__main__":
    if len(sys.argv)== 2:
        input_file : str = sys.argv[1]
        output_file = f"./tests/{input_file.split('.')[0]}_coreness.txt"
    else:
        sys.exit(1)
    main(input_file, output_file)
