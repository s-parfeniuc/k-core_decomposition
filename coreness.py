#!/usr/bin/env python
import networkx as nx
import sys

def read_graph_from_file(file_path):
    """Legge il grafo da un file, restituendo un grafo di NetworkX"""
    G = nx.Graph()
    with open(file_path, 'r') as file:
        for line in file:
            if line.startswith("#"):
                continue
            # Assumiamo che ogni riga contenga due nodi separati da spazio
            node1, node2 = map(int, line.split())
            G.add_edge(node1, node2)
    return G

def calculate_coreness(G):
    """Calcola la coreness di ogni nodo nel grafo"""
    # Calcoliamo la coreness con il metodo k-core decomposition di NetworkX
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
    # 1. Leggi il grafo dal file
    G = read_graph_from_file(input_file)
    
    # 2. Calcola la coreness
    coreness = calculate_coreness(G)
    
    # 3. Salva il risultato su un file
    save_coreness_to_file(coreness, output_file)
    print(f"Coreness salvata in {output_file}")

if __name__ == "__main__":
    # Sostituisci questi con i percorsi reali dei tuoi file
    if len(sys.argv)== 2:
        input_file : str = sys.argv[1]
        output_file = f"./tests/{input_file.split('.')[0]}_coreness.txt"
    else:
        sys.exit(1)
    main(input_file, output_file)
