use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use std::cmp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::usize::MAX;
/*
Versione "multi-threaded" che usa gli iteratori paralleli di rayon. L'accesso ad ogni nodo è gestito
da una mutex. Ogni nodo è composto dai propri dati locali e una lista di messaggi di tipo
MessageMail, che è l'unico punto di interazione tra nodi diversi. MessageMail deve quindi essere
un ReferenceCount (atomic) e acceduto tramite mutex, dato che ogni thread ha i riferimenti
a tutti i MessageMail dei propri vicini (e non dei nodi stessi, in modo da non dover bloccare
l'accesso a un intero nodo solo per scrivere un messaggio).
*/

struct MessageMail {
    id: usize,
    messages: VecDeque<(usize, usize)>,
}

impl MessageMail {
    pub fn new(n: usize) -> Self {
        Self {
            id: n,
            messages: VecDeque::new(),
        }
    }

    pub fn push(&mut self, message: (usize, usize)) {
        self.messages.push_back(message);
    }

    pub fn pop(&mut self) -> Option<(usize, usize)> {
        self.messages.pop_back()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

struct Node {
    id: usize,
    coreness: usize,
    est: HashMap<usize, usize>, // est[k] è l'estimate locale della coreness del vicino che ha id k,
    neighbors: Vec<Arc<Mutex<MessageMail>>>, // lista di Message_Mail dei vicini,
    messages: Arc<Mutex<MessageMail>>, // queue di messaggi
    changed: bool,              // indica se è stato cambiato dall'ultima fase di invio messaggi
}

impl Node {
    pub fn new(n: usize) -> Self {
        Node {
            id: n,
            coreness: MAX,
            est: HashMap::new(),
            neighbors: Vec::new(),
            messages: Arc::new(Mutex::new(MessageMail::new(n))),
            changed: true,
        }
    }

    pub fn add_neighbor(&mut self, node: Arc<Mutex<MessageMail>>) {
        // aggiunge un nuovo vicino
        self.neighbors.push(node);
    }

    pub fn init(&mut self) {
        // elimina duplicati, se presenti, dalla lista di vicini
        let mut seen: HashSet<usize> = HashSet::new();

        // usa un HashSet per vedere se gli elementi sono già stati "visti"
        self.neighbors.retain(|x| {
            let id = x.lock().unwrap().id;
            seen.insert(id) // il metodo restituisce true se l'id non è stato ancora inserito
        });

        // inizializza gli estimates dei vicini a "infinito"
        for neighbor in &self.neighbors {
            self.est.insert(neighbor.lock().unwrap().id, MAX);
        }

        // inizializza la propria coreness con la lunghezza della lista di adiacenza
        self.coreness = self.neighbors.len();

        // invia un messaggio a tutti i vicini
        self.send_messages();
    }

    pub fn compute_index(&mut self) -> usize {
        let mut messages = self.messages.lock().unwrap();
        // se la lista di messaggi è vuota non devo calcolare coreness
        if messages.is_empty() {
            return self.coreness;
        }
        // controllo la lista di messaggi e aggiorno gli estimates locali
        while !messages.is_empty() {
            let message = messages.pop().unwrap();
            if message.1 < *self.est.get(&message.0).unwrap() {
                self.est.insert(message.0, message.1);
            }
        }

        let core = self.coreness;

        if core == 0 {
            return 0;
        }

        let mut count: Vec<usize> = Vec::with_capacity(core + 1);
        count.resize(core + 1, 0);

        for neighbor in &self.est {
            let k = cmp::min(core, *neighbor.1);
            count[k] += 1;
        }

        for i in (1..core + 1).rev() {
            count[i - 1] += count[i];
        }

        let mut i = core;
        while i > 1 && count[i] < i {
            i -= 1;
        }
        if i < core {
            self.changed = true;
        }
        self.coreness = i;

        return i;
    }

    pub fn send_messages(&mut self) {
        if self.changed {
            self.changed = false;
            for neighbor in &mut self.neighbors {
                neighbor
                    .lock()
                    .unwrap()
                    .messages
                    .push_back((self.id, self.coreness));
            }
        }
    }
}

struct Graph {
    inmap: Vec<Node>,
}

impl Graph {
    pub fn new() -> Self {
        Graph { inmap: Vec::new() }
    }

    pub fn add_edge(&mut self, i: usize, j: usize) {
        if i >= self.inmap.len() || j >= self.inmap.len() {
            let old_len = self.inmap.len();
            let new_len = cmp::max(i, j) + 1;
            for n in old_len..new_len {
                self.inmap.push(Node::new(n));
            }
        }
        let message_j = Arc::clone(&self.inmap[j].messages);
        let message_i = Arc::clone(&self.inmap[i].messages);

        self.inmap[i].add_neighbor(Arc::clone(&message_j));
        self.inmap[j].add_neighbor(Arc::clone(&message_i));
    }

    // inizializza tutti i nodi con i valori opportuni, chiamata dopo aver aggiunto tutti i nodi
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
                let _ = self.add_edge(numbers[0], numbers[1]);
            } else {
                println!("Skipping invalid line: {}", line);
            }
        }
        Ok(())
    }

    pub fn write_to_file(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        for n in &self.inmap {
            writeln!(file, "{}", n.coreness)?;
        }

        Ok(())
    }

    pub fn compute_coreness_threadpool(&mut self) {
        let pool = rayon::ThreadPoolBuilder::new().build().unwrap();
        let cont_mutex = Mutex::new(true);
        let guard = cont_mutex.lock().unwrap();
        let mut cont = *guard;

        drop(guard);
        while cont {
            let mut guard = cont_mutex.lock().unwrap();
            *guard = false;
            drop(guard);
            pool.scope(|scope| {
                for node in self.inmap.iter_mut() {
                    scope.spawn(|_| {
                        node.compute_index();
                        if node.changed {
                            *cont_mutex.lock().unwrap() = true;
                        }
                    });
                }
            });
            pool.scope(|scope| {
                for node in self.inmap.iter_mut() {
                    if node.changed {
                        node.changed = false;
                        scope.spawn(|_| {
                            for neighbor in node.neighbors.iter_mut() {
                                neighbor
                                    .lock()
                                    .unwrap()
                                    .messages
                                    .push_back((node.id, node.coreness));
                            }
                        });
                    }
                }
            });
            let guard = cont_mutex.lock().unwrap();
            cont = *guard;
            drop(guard);
        }
    }

    pub fn compute_coreness(&mut self) {
        let mut cont = true;

        let mut rounds = 0;

        while cont {
            rounds += 1;

            // computazione in parallelo dei coreness dei nodi
            cont = self
                .inmap
                .par_iter_mut() // itera in modo parallelo il vettore
                .map(|x| thread_routine_index(x)) // applica la funzione thread_routine_index a ogni elemento
                .reduce(|| false, |a, b| a || b); // or tra tutti i risultati: se almeno uno è stato cambiato si continua

            // invio messaggi a vicini
            self.inmap
                .par_iter_mut() // itera in modo parallelo il vettore
                .for_each(|x| thread_routine_message(x)) //applica la funzione thread_routine_message a ogni
        }
        println!("Algoritmo terminato dopo {} iterazioni", rounds);
    }
}

// funzione eseguita dai thread in fase di computazione coreness su ogni nodo
fn thread_routine_index(node: &mut Node) -> bool {
    node.compute_index();
    return node.changed;
}

// funzione eseguita dai thread in fase di invio messaggi
fn thread_routine_message(node: &mut Node) {
    // prende le lock dei MessageMail dei vicini, non dei nodi stessi, così c'è contention solo per l'accesso
    // alla lista di messaggi, non sono possibili deadlock perché si acquisisce una lock alla volta e si
    // rilascia a fine operazione
    if node.changed {
        node.changed = false;
        for neighbor in &node.neighbors {
            neighbor.lock().unwrap().push((node.id, node.coreness));
        }
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

    // crea il grafo a partire dal file
    graph.parse_file(in_file)?;

    // inizializza le variabili necessarie per l'algoritmo
    graph.init_graph();

    // algoritmo di calcolo coreness dei nodi
    graph.compute_coreness_threadpool();

    // scrittura valori di coreness dei nodi
    graph.write_to_file(&out_file)?;

    Ok(())
}
