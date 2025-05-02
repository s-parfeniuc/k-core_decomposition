# Test
Per testare la correttezza di una versione:   
```
    cargo run --bin=version ./graphs/nome_grafo/nome_grafo.txt ./graphs/nome_grafo/out.txt  
```
e poi fare  
```
    diff out.txt networkx.txt  
```
nella cartella del grafo: networkx.txt è il risultato della libreria networkx.  

## Versioni

1. main.rs - prima versione single-threaded, senza struct dei nodi e con una coda globale dei nodi da aggiornare.
2. bin/01.rs - versione single-threaded che simula il comportamento dei nodi in una rete che comunicano tra di loro in assenza di canale broadcast.
3. bin/02.rs - versione parallelizzata di bin/01.rs.
4. bin/03.rs - al posto di utilizzare hashmap per gli estimates dei vicini vengono usati dei vettori ordinati.
5. bin/04.rs - ogni iterazione non è più divisa in 2 parti ("lettura" e "scrittura"), i nodi leggono e scrivono contemporaneamente
6. bin/05.rs - aggiunta ottimizzazione descritta nel paper: scrittura messaggi solo se la propria coreness è minore di quella del vicino
7. bin/06.rs - versione one host - multiple nodes
