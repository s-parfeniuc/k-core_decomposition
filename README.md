## Utilizzo
Tutte le versioni prendono 2 argomenti da linea di comando: file di input (in formato .txt in cui ogni riga rappresenta un arco) e file di output (scrive solo la coreness dei nodi in ordine di indice).

## Versioni

1. main.rs - prima versione single-threaded, senza struct dei nodi e con una coda (prioritaria) globale dei nodi da aggiornare.
2. bin/00.rs - FastK, versione centralizzata parallela che utilizza strutture condivise lock-free. Versione ibrida che utilizza parallelismo più coda prioritaria superata una soglia di numeri attivi
3. bin/01.rs - SequentialK, versione single-threaded che simula il comportamento dei nodi in una rete che comunicano tra di loro in assenza di canale broadcast.
4. bin/02.rs - Versione parallelizzata di bin/01.rs.
5. bin/03.rs - al posto di utilizzare hashmap per gli estimates dei vicini vengono usati dei vettori ordinati.
6. bin/04.rs - ogni iterazione non è più divisa in 2 parti ("lettura" e "scrittura"), i nodi leggono e scrivono contemporaneamente
7. bin/05.rs - ParallelK, aggiunta ottimizzazione descritta nel paper: scrittura messaggi solo se la propria coreness è minore di quella del vicino
8. bin/06.rs e bin/07.rs - Versione one host - multiple nodes: partizionamento del grafo e comunicazione stime intra round. Presenta problemi di sbilanciamento del carico di lavoro tra sottografi, mostra l'importanza di una buona euristica di partizionamento.
9. bin/08.rs - i nodi hanno riferimenti diretti alle coreness dei vicini (AtomicUsize). Utilizzo di una coda concorrente, batched e prioritaria in base al logaritmo della coreness stimata del vicino.