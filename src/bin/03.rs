use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::cmp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::usize::MAX;
/*

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
    times: f64,
}

impl Node {
    pub fn new(n: usize) -> Self {
        Node {
            times: 0.0,
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

        // inizializza la proprio coreness con la lunghezza della lista di adiacenza
        self.coreness = self.neighbors.len();

        // invia un messaggio a tutti i vicini
        self.send_messages();
    }

    pub fn compute_index(&mut self) -> usize {
        let mut messages = self.messages.lock().unwrap();
        // se la lista di messaggi non devo calcolare coreness
        if messages.is_empty() {
            return self.coreness;
        }
        self.times += 1.0;
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

    pub fn receive_message(&mut self, message: (usize, usize)) {
        self.messages.lock().unwrap().messages.push_back(message);
    }
}

struct Graph {
    inmap: Vec<Arc<Mutex<Node>>>,
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
                self.inmap.push(Arc::new(Mutex::new(Node::new(n))));
            }
        }
        self.inmap[i]
            .lock()
            .unwrap()
            .add_neighbor(Arc::clone(&self.inmap[j].lock().unwrap().messages));
        self.inmap[j]
            .lock()
            .unwrap()
            .add_neighbor(Arc::clone(&self.inmap[i].lock().unwrap().messages));
    }

    // inizializza tutti i nodi con i valori opportuni, chiamata dopo aver aggiunto tutti i nodi
    pub fn init_graph(&mut self) {
        for node in &self.inmap {
            node.lock().unwrap().init();
        }
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
            writeln!(file, "{}", n.lock().unwrap().coreness)?;
        }

        Ok(())
    }

    pub fn compute_coreness(&mut self) {
        let mut cont = true;

        let mut rounds = 0;
        let mut n_messages = 0;

        while cont {
            rounds += 1;

            // computazione in parallelo dei coreness dei nodi
            cont = self
                .inmap
                .par_iter() // itera in modo parallelo il vettore
                .map(|x| thread_routine_index(x.clone())) // applica la funzione thread_routine_index a ogni elemento
                .reduce(|| false, |a, b| a || b); // or tra tutti i risultati: se almeno uno è stato cambiato si continua

            // invio messaggi a vicini
            n_messages += self
                .inmap
                .par_iter() // itera in modo parallelo il vettore
                .map(|x| thread_routine_message(x.clone())) //applica la funzione thread_routine_message a ogni
                .reduce(|| 0, |a, b| a + b); // sommatoria del numero di messaggi inviati in questa iterazione
        }
        println!(
            "Algoritmo terminato dopo {} iterazioni, numero di messaggi scambiati: {}",
            rounds, n_messages
        );
        let mut avg: f64 = 0.0;
        for node in &self.inmap {
            let locked_node = node.lock().unwrap();
            avg += locked_node.times;
        }
        println!(
            "In media ogni nodo ha calcolato la propria coreness {} volte",
            avg / (self.inmap.len() - 1) as f64
        );
    }
}

// funzione eseguita dai thread in fase di computazione coreness su ogni nodo
fn thread_routine_index(node: Arc<Mutex<Node>>) -> bool {
    let mut locked_node = node.lock().unwrap();
    locked_node.compute_index();
    return locked_node.changed;
}

// funzione eseguita dai thread in fase di invio messaggi
// non è mai in possesso di 2 lock diverse: crea una copia locale dei riferimenti ai vicini e rilascia la lock del nodo
fn thread_routine_message(node: Arc<Mutex<Node>>) -> usize {
    // crea una copia locale di riferimenti ai vicini e del proprio messaggio da inviare
    let mut locked_node = node.lock().unwrap();
    if locked_node.changed {
        locked_node.changed = false;
        for neighbor in &locked_node.neighbors {
            neighbor
                .lock()
                .unwrap()
                .push((locked_node.id, locked_node.coreness));
        }
        return locked_node.neighbors.len();
    }
    return 0;
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
    graph.compute_coreness();

    // scrittura valori di coreness dei nodi
    graph.write_to_file(&out_file)?;

    Ok(())
}
