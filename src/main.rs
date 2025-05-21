#![allow(dead_code)]
use std::cmp;
use std::cmp::max;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::time::Instant;

struct Graph {
    inmap: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph { inmap: Vec::new() }
    }

    fn add_edge(&mut self, i: usize, j: usize) -> Result<(), ()> {
        let max = max(i, j) + 1;
        if self.inmap.len() < max {
            self.inmap.resize(max, Vec::new());
        }

        self.inmap[i].push(j);
        self.inmap[j].push(i);
        Ok(())
    }

    pub fn no_duplicates(&mut self) {
        for i in 0..self.inmap.len() {
            self.inmap[i].sort();
            let mut n: usize = 0;
            for j in 1..self.inmap[i].len() {
                if self.inmap[i][n] != self.inmap[i][j] {
                    n += 1;
                    self.inmap[i][n] = self.inmap[i][j];
                }
            }
            self.inmap[i].truncate(n + 1);
        }
    }
}

pub struct Queue {
    queues: Vec<VecDeque<usize>>,
}

impl Queue {
    pub fn new(max_k: usize) -> Self {
        let mut queues = Vec::with_capacity(max_k + 1);
        for _ in 0..=max_k {
            queues.push(VecDeque::new());
        }
        Self { queues }
    }

    /// Inserisce un elemento `value` nella coda di priorità `k`
    pub fn push(&mut self, k: usize, value: usize) {
        if k < self.queues.len() {
            self.queues[k].push_back(value);
        } else {
            panic!("Priorità fuori dai limiti: {}", k);
        }
    }

    /// Estrae il primo elemento disponibile dalla coda di priorità più alta (cioè dal primo indice non vuoto)
    pub fn pop(&mut self) -> Option<usize> {
        for queue in &mut self.queues {
            if let Some(value) = queue.pop_front() {
                return Some(value);
            }
        }
        None
    }

    /// Controlla se tutte le code sono vuote
    pub fn is_empty(&self) -> bool {
        self.queues.iter().all(|q| q.is_empty())
    }
}

struct Data {
    graph: Graph,
    est: Vec<usize>,
    changed: Vec<bool>,
    count: Vec<usize>,
    queue: VecDeque<usize>,
}

impl Data {
    pub fn new(graph: Graph) -> Self {
        let mut est: Vec<usize> = Vec::with_capacity(graph.inmap.len());
        let mut changed: Vec<bool> = Vec::with_capacity(graph.inmap.len());

        for i in 0..graph.inmap.len() {
            est.push(graph.inmap[i].len());
            changed.push(false);
        }
        Data {
            graph: graph,
            est: est,
            changed: changed,
            count: Vec::new(),
            queue: VecDeque::new(),
        }
    }
}

fn compute_index(coreness: &mut Data, u: usize) -> usize {
    let core = coreness.est[u];

    if core == 0 {
        return 0;
    }
    coreness.count.resize(core + 1, 0);

    for neighbor in &coreness.graph.inmap[u] {
        let k = cmp::min(core, coreness.est[*neighbor]);
        coreness.count[k] += 1;
    }

    for i in (1..core + 1).rev() {
        coreness.count[i - 1] += coreness.count[i];
    }
    let mut i = core;
    while i > 1 && coreness.count[i] < i {
        i -= 1;
    }
    coreness.count.clear();
    return i;
}

fn compute_coreness_queue(core: &mut Data) {
    let max_est = *core.est.iter().max().unwrap_or(&1); // evita log2(0)
    let max_k = (max_est as f64).log2().floor() as usize;
    let mut queue = Queue::new(max_k);
    for i in 0..core.graph.inmap.len() {
        queue.push(0, i);
    }
    let mut elaborazioni = 0;
    let mut elaborazioni_vere = 0;
    let mut vicini_visti = 0;
    while !queue.is_empty() {
        if let Some(node) = queue.pop() {
            elaborazioni += 1;
            core.changed[node] = false;
            let old_estimate = core.est[node];
            let new_estimate = compute_index(core, node);

            if new_estimate < old_estimate {
                elaborazioni_vere += 1;
                vicini_visti += &core.graph.inmap[node].len();
                for i in &core.graph.inmap[node] {
                    if !core.changed[*i]
                        && new_estimate < core.est[*i]
                        && old_estimate >= core.est[*i]
                    {
                        let priority = (core.est[*i] as f64).log2().floor() as usize;
                        queue.push(priority, *i);
                        core.changed[*i] = true;
                    }
                }
                core.est[node] = new_estimate;
            }
        }
    }
    println!("Nodi elaborati {} volte in tutto", elaborazioni);
    println!("Nodi cambiati {} volte in tutto", elaborazioni_vere);
    println!("Vicini visti {}", vicini_visti);
}

fn compute_coreness_queue_normal(core: &mut Data) {
    for i in 0..core.graph.inmap.len() {
        core.queue.push_back(i);
    }
    let mut elaborazioni = 0;
    let mut elaborazioni_vere = 0;
    let mut vicini_visti = 0;
    while !core.queue.is_empty() {
        if let Some(node) = core.queue.pop_front() {
            elaborazioni += 1;
            core.changed[node] = false;
            let old_estimate = core.est[node];
            let new_estimate = compute_index(core, node);

            if new_estimate < old_estimate {
                elaborazioni_vere += 1;
                vicini_visti += &core.graph.inmap[node].len();
                for i in &core.graph.inmap[node] {
                    if !core.changed[*i]
                        && new_estimate < core.est[*i]
                        && old_estimate >= core.est[*i]
                    {
                        core.queue.push_back(*i);
                        core.changed[*i] = true;
                    }
                }
                core.est[node] = new_estimate;
            }
        }
    }
    println!(
        "Nodi elaborati {} volte in tutto senza priority",
        elaborazioni
    );
    println!("Nodi cambiati {} volte in tutto", elaborazioni_vere);
    println!("Vicini visti {}", vicini_visti);
}

fn compute_coreness(core: &mut Data) {
    let mut continua = true;
    while continua {
        continua = false;

        for i in 0..core.graph.inmap.len() {
            let new_estimate = compute_index(core, i);
            if new_estimate < core.est[i] {
                core.est[i] = new_estimate;
                core.changed[i] = true;
                continua = true;
            }
        }
    }
}

fn write_to_file(vec: Vec<usize>, filename: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(filename)?;
    let mut start = 0;
    if vec[0] == 0 {
        start = 1;
    }
    for n in start..vec.len() {
        writeln!(file, "{}", vec[n])?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let file_path = "./graphs/soc-pokec-relationships/soc-pokec-relationships.txt";

    let mut graph = Graph::new();

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut start = Instant::now();
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
            let _ = graph.add_edge(numbers[0], numbers[1]);
        } else {
            println!("Skipping invalid line: {}", line);
        }
    }
    println!("Per parsare il file: {:?}", start.elapsed());
    start = Instant::now();
    graph.no_duplicates();

    println!("Per inizializzare i nodi: {:?}", start.elapsed());
    start = Instant::now();
    let mut algorithm: Data = Data::new(graph);

    compute_coreness_queue(&mut algorithm);

    //compute_coreness_queue_normal(&mut algorithm);

    println!("Per calcolare coreness version01: {:?}", start.elapsed());

    let _ = write_to_file(algorithm.est, "./graphs/soc-pokec-relationships/coree.txt");

    Ok(())
}
