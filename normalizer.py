#!/home/tesi/version01/.venv/bin/python
import sys
def convert_to_undirected_edgelist(input_file, output_file):
    edges = set()
    nodes = set()

    with open(input_file, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('#') or not line:
                continue
            parts = line.split()
            if len(parts) < 2:
                continue
            u, v = map(int, parts[:2])
            if u == v:
                continue
            # Normalizziamo l'ordine per evitare duplicati (min, max)
            edge = tuple(sorted((u, v)))
            edges.add(edge)
            nodes.update(edge)

    # Rinomina i nodi da 0 in poi
    sorted_nodes = sorted(nodes)
    node_map = {node: idx for idx, node in enumerate(sorted_nodes)}

    with open(output_file, 'w') as f_out:
        for u, v in sorted(edges):
            f_out.write(f"{node_map[u]} {node_map[v]}\n")

    print(f"Creato file '{output_file}' con {len(edges)} archi e {len(node_map)} nodi.")

# Esempio di utilizzo
if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Uso: python normalizer.py <input_file> <output_file>")
        sys.exit(1)
    # Chiamata alla funzione con i parametri da riga di comando
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    # Converti il file di input in un formato non diretto
    # e salva il risultato nel file di output
    convert_to_undirected_edgelist(input_file, output_file)
