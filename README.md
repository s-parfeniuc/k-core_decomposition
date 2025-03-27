# Test
Per testare la correttezza di una versione: 
cargo run --bin=version ./graphs/nome_grafo/nome_grafo.txt ./graphs/nome_grafo/out.txt
e poi fare 
diff out.txt networkx.txt
nella cartella del grafo: networkx.txt è il risultato della libreria networkx.