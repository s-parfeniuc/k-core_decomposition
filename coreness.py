#!/home/tesi/version01/.venv/bin/python
import networkx as nx
import sys
import time
import csv

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

def main():
    data_filename = "data/networkx.csv"
    graphs = [
        "p2p-Gnutella08",
        "web-Stanford",
        "web-BerkStan",
        "web-Google",
        "web-NotreDame",
        "wiki-Talk",
        "soc-pokec-relationships",
        "soc-LiveJournal1",
        "roadNet-CA",
        "roadNet-PA",
        "roadNet-TX",
    ]

    with open(data_filename, mode='a', newline='') as file:
        writer = csv.writer(file)
        
        for graph_name in graphs:
            input_file = f"graphs/{graph_name}/{graph_name}.txt"

            G = read_graph_from_file(input_file)
            G.remove_edges_from(nx.selfloop_edges(G))

            start_time = time.time()  
            coreness = calculate_coreness(G) 
            end_time = time.time() 

            print(graph_name + ", runtime: " + str(end_time - start_time) + "s")
            
#            for _ in range(5):
#                start_time = time.time()  
#                coreness = calculate_coreness(G) 
#                end_time = time.time() 
#                try:
#                    execution_time = end_time - start_time  # tempo di esecuzione
#                except:
#                    execution_time = "0s"
#                writer.writerow([graph_name, str(execution_time) + "s"])
                
            
    

if __name__ == "__main__":
    main()


