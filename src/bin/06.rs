#![allow(dead_code)]
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use std::cmp;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::sync::Barrier;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::usize::MAX;
/*
Versione in cui al posto di usare messaggi uno a uno tra nodi, si usa un singolo messaggio broadcast a cui ogni nodo può accedere,
esso viene aggiornato alla fine di ogni iterazione.
*/

/*
Ogni thread ha un sottografo disgiunto con quello degli altri. Insieme a questo, ha un array est, che rappresenta i valori coreness
di tutti i nodi del proprio sottografo e dei nodi vicini ai propri.
*/
struct Node {
    neighbors: Vec<usize>, // lista degli indici dei vicini
    in_queue: bool, // indica se è nella lista dei nodi che devono ricalcolare la propria coreness
    changed: bool,  // indica se è cambiato dall'ultima iterazione
}

impl Node {
    pub fn new() -> Self {
        Node {
            neighbors: Vec::new(),
            in_queue: false,
            changed: true,
        }
    }

    pub fn add_neighbor(&mut self, id: usize) {
        self.neighbors.push(id);
    }

    pub fn init(&mut self) {
        // eliminiamo i duplicati dall lista di adiacenza
        self.neighbors.sort();
        let mut j = 0;
        for i in 1..self.neighbors.len() {
            if self.neighbors[i] != self.neighbors[j] {
                j += 1;
                self.neighbors[j] = self.neighbors[i];
            }
        }
        self.neighbors.truncate(j + 1);

        self.changed = true;
    }

    pub fn compute_index(
        &mut self,
        coreness: &Vec<usize>, // estimates dei nodi dello stesso sottografo
        est: &std::sync::RwLockReadGuard<'_, Vec<usize>>, // estimates dei nodi esterni al sottografo
        core: &usize,
        start: &usize,
        end: &usize,
    ) -> usize {
        if *core == 0 {
            return 0;
        }

        let mut count: Vec<usize> = Vec::with_capacity(core + 1);
        count.resize(core + 1, 0);

        for neighbor in &self.neighbors {
            let k;
            if *neighbor >= *start && *neighbor < *end {
                k = cmp::min(*core, coreness[*neighbor - *start]);
            } else {
                k = cmp::min(*core, est[*neighbor]);
            }

            count[k] += 1;
        }

        for i in (1..core + 1).rev() {
            count[i - 1] += count[i];
        }

        let mut i = *core;
        while i > 1 && count[i] < i {
            i -= 1;
        }

        if i < *core {
            self.changed = true;
            self.in_queue = false;
        }
        return i;
    }
}

struct Graph {
    inmap: Vec<Node>,
    est: Vec<usize>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            inmap: Vec::new(),
            est: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, i: usize, j: usize) {
        if i >= self.inmap.len() || j >= self.inmap.len() {
            let old_len = self.inmap.len();
            let new_len = cmp::max(i, j) + 1;
            for _n in old_len..new_len {
                self.inmap.push(Node::new());
            }
        }
        if i == j {
            return;
        }

        self.inmap[i].add_neighbor(j);
        self.inmap[j].add_neighbor(i);
    }

    // inizializza tutti i nodi con i valori opportuni, crea il "messaggio broadcast" che verrà condiviso tra i nodi
    // chiamata dopo aver aggiunto tutti i nodi
    pub fn init_graph(&mut self) {
        self.inmap.par_iter_mut().for_each(|x| {
            x.init();
        });
    }

    // crea un grafo a partire da un file ignorando righe che iniziano con #
    pub fn parse_file(&mut self, filename: &str) -> std::io::Result<()> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') {
                continue;
            }
            let numbers: Vec<usize> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if numbers.len() == 2 {
                self.add_edge(numbers[0], numbers[1]);
            } else {
                println!("Linea non valida: {}", line);
            }
        }
        Ok(())
    }

    // metodo che scrive su un file il valore di coreness di ciascun nodo
    pub fn write_to_file(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        for n in &self.est {
            writeln!(file, "{}", *n)?;
        }

        Ok(())
    }

    pub fn compute_coreness_threads(&mut self, num_threads: &usize, _chunk_size: &usize) -> usize {
        let mut iter: usize = 0;
        thread::scope(|scope| {
            // divido il grafo in num_threads sottografi uguali, protette da Mutex
            let mut subgraphs: Vec<Mutex<&mut [Node]>> = Vec::new();

            let len = self.inmap.len();

            for chunk in self.inmap.chunks_mut((len / *num_threads) + 1) {
                subgraphs.push(Mutex::new(chunk));
            }

            //
            let mut messages: Vec<usize> = Vec::new();
            messages.resize(len, MAX);

            for subgraph in subgraphs.iter() {
                let len = subgraph.lock().unwrap().len();
                let mut vec = Vec::with_capacity(len);
                vec.resize_with(len, || MAX);
            }
            // Preparo altre strutture e variabili necessarie per la sincronizzazione tra i thread
            let cont_mutex: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
            let barrier = Arc::new(Barrier::new(*num_threads + 1));
            let mut threads = Vec::with_capacity(*num_threads);

            let shared_messages: Arc<RwLock<Vec<usize>>> = Arc::new(RwLock::new(messages));

            let shared_subgraphs: Arc<Vec<Mutex<&mut [Node]>>> = Arc::new(subgraphs);

            let mut start = 0;
            for i in 0..*num_threads {
                let len = shared_subgraphs[i].lock().unwrap().len();
                let cont_clone = Arc::clone(&cont_mutex);
                let barrier_clone = Arc::clone(&barrier);
                let subgraph_clone = Arc::clone(&shared_subgraphs);
                let messages_clone = Arc::clone(&shared_messages);

                threads.push(scope.spawn(move || {
                    thread_routine(
                        cont_clone,
                        barrier_clone,
                        i,
                        start,
                        subgraph_clone,
                        messages_clone,
                    );
                }));
                start += len;
            }

            let mut continua = true;

            while continua {
                iter += 1;
                let mut cont_guard = cont_mutex.lock().unwrap();
                *cont_guard = false;
                drop(cont_guard);
                // prima fase

                barrier.wait();

                // seconda fase

                barrier.wait();
                let cont_guard = cont_mutex.lock().unwrap();
                continua = *cont_guard;
                drop(cont_guard);
                barrier.wait();
            }

            for &coreness in shared_messages.read().unwrap().iter() {
                self.est.push(coreness);
            }
        });
        return iter;
    }
}

fn thread_routine(
    cont: Arc<Mutex<bool>>,
    barrier: Arc<Barrier>,
    index: usize,
    start: usize,
    subgraphs: Arc<Vec<Mutex<&mut [Node]>>>,
    shared_messages: Arc<RwLock<Vec<usize>>>,
) {
    let mut continua = true;
    let mut subgraph: std::sync::MutexGuard<'_, &mut [Node]> = subgraphs[index].lock().unwrap();
    let len = subgraph.len();

    let end = len + start;

    let mut local_estimate: Vec<usize> = subgraph.iter().map(|x| x.neighbors.len()).collect();

    while continua {
        continua = false;
        let est = shared_messages.read().unwrap();
        let mut nodes_to_compute: VecDeque<usize> = (0..local_estimate.len()).collect(); // metto tutti i nodi in coda per aggiornare la coreness
        while !nodes_to_compute.is_empty() {
            let node_index = nodes_to_compute.pop_front().unwrap();
            subgraph[node_index].in_queue = false;

            let old_coreness = local_estimate[node_index];

            let new_coreness = subgraph[node_index].compute_index(
                &local_estimate,
                &est,
                &old_coreness,
                &start,
                &end,
            );

            local_estimate[node_index] = new_coreness;

            if old_coreness > new_coreness {
                continua = true;

                // prendo la lista di vicini da ricalcolare
                let neighbors_to_modify: Vec<usize> = subgraph[node_index]
                    .neighbors
                    .iter()
                    .copied()
                    .filter(|&neighbor_index| neighbor_index >= start && neighbor_index < end)
                    .collect(); // seleziono i nodi che appartengono al mio sottografo

                for &neighbor_index in neighbors_to_modify.iter() {
                    let local_index = neighbor_index - start;
                    // se il vicino appartiene al mio sottografo
                    if !subgraph[local_index].in_queue
                        && local_estimate[local_index] > new_coreness
                        && old_coreness >= local_estimate[local_index]
                    {
                        // se il vicino non è in coda
                        subgraph[local_index].in_queue = true;
                        nodes_to_compute.push_back(local_index);
                    }
                }
            }
        }
        drop(est);

        // fine prima fase, sincronizzo con altri thread
        barrier.wait();

        // seconda fase, invio messaggi
        let mut write_lock = shared_messages.write().unwrap();

        for node_index in 0..subgraph.len() {
            if subgraph[node_index].changed {
                subgraph[node_index].changed = false;
                let message_index = node_index + start;
                write_lock[message_index] = local_estimate[node_index];
            }
        }
        drop(write_lock);
        let mut cont_guard = cont.lock().unwrap();
        *cont_guard |= continua;
        drop(cont_guard);
        barrier.wait();
        let cont_guard = cont.lock().unwrap();
        continua = *cont_guard;
        drop(cont_guard);
        barrier.wait();
    }
}

fn main() -> std::io::Result<()> {
    // gestione argomenti passati da linea di comando: in_file (file da parsare) e out_file (file su cui scrivere la coreness dei nodi)
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("2 argomenti: file di input e file in cui scrivere");
        return Ok(());
    }
    let in_file = &args[1];
    let out_file = &args[2];

    let mut graph = Graph::new();

    let mut start = Instant::now();

    // crea il grafo a partire dal file
    graph.parse_file(in_file)?;
    println!("Per parsare il file: {:?}", start.elapsed());

    // inizializza le variabili necessarie per l'algoritmo
    start = Instant::now();
    graph.init_graph();
    println!("Per inizializzare i nodi: {:?}", start.elapsed());

    // calcolo coreness tramite threads
    start = Instant::now();
    let iter = graph.compute_coreness_threads(&6, &256);
    println!(
        "Per calcolare coreness con threads: {:?}; in {} iterazioni",
        start.elapsed(),
        iter
    );

    // scrittura valori di coreness dei nodi
    graph.write_to_file(&out_file)?;

    Ok(())
}
