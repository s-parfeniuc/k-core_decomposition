# Test
Per testare la correttezza di una versione: 
'cargo run --bin=version ./graphs/nome_grafo/nome_grafo.txt ./graphs/nome_grafo/out.txt'
e poi fare 
'diff out.txt networkx.txt'
nella cartella del grafo: networkx.txt è il risultato della libreria networkx.
## Versioni
1. main.rs - prima versione single-threaded, senza struct dei nodi e con una coda globale dei nodi da aggiornare.
2. bin/01.rs - versione single-threaded che simula il comportamento dei nodi in una rete che comunicano tra di loro in assenza di canale broadcast.
3. bin/02.rs - versione parallelizzata di bin/01.rs, che usa gli iteratori paralleli della libreria rayon.