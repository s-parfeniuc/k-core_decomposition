use std::cmp;
use std::cmp::max;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead};

struct Graph {
    inmap: Vec<Vec<usize>>,
    out: usize,
    n_allocs: u32,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            inmap: Vec::new(),
            out: 0,
            n_allocs: 0,
        }
    }

    fn add_edge(&mut self, i: usize, j: usize) -> Result<(), ()> {
        let max = max(i, j) + 1;
        if self.inmap.len() < max {
            self.inmap.resize(max, Vec::new());
            self.n_allocs += 1;
        }

        self.inmap[i].push(j);
        self.inmap[j].push(i);
        self.out += 1;
        Ok(())
    }

    pub fn debug_print(&mut self) {
        println!("Number of nodes: {}", self.inmap.len());
        println!("Number of edges: {}", self.out);
        println!("Number of resizes: {}", self.n_allocs);
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

    pub fn print(&mut self) {
        println!("Number of edges: {}", self.out);
        for i in 0..self.inmap.len() {
            for j in &self.inmap[i] {
                println!("{} <--> {}", i, j);
            }
        }
    }
}

struct Data {
    graph: Graph,
    est: Vec<usize>,
    changed: Vec<bool>,
    count: Vec<Vec<usize>>,
    queues: [VecDeque<(usize, usize)>; 2], //all'iterazione i uso i%2 come coda attuale e i+1%2 come coda per la prossima iterazione
}

impl Data {
    pub fn new(graph: Graph) -> Self {
        let mut est: Vec<usize> = Vec::with_capacity(graph.inmap.len());
        let mut changed: Vec<bool> = Vec::with_capacity(graph.inmap.len());
        let mut count: Vec<Vec<usize>> = Vec::with_capacity(graph.inmap.len());

        for i in 0..graph.inmap.len() {
            est.push(graph.inmap[i].len());
            changed.push(false);
            count.push(Vec::with_capacity(est[i]));
        }
        Data {
            graph: graph,
            est: est,
            changed: changed,
            count: count,
            queues: [VecDeque::new(), VecDeque::new()],
        }
    }
}

fn compute_index(coreness: &mut Data, u: usize) -> usize {
    let core = coreness.est[u];
    coreness.count[u].resize(core, 0);

    for neighbor in &coreness.graph.inmap[u] {
        let k = cmp::min(core - 1, coreness.est[*neighbor] - 1);
        coreness.count[u][k] += 1;
    }
    for i in (1..core - 1).rev() {
        coreness.count[u][i - 1] += coreness.count[u][i];
    }
    let mut i = core - 1;
    while i > 0 && coreness.count[u][i] < i {
        i -= 1;
    }
    coreness.count[u].clear();
    return i + 1;
}

fn compute_coreness(core: &mut Data) {
    let mut iteration: usize = 0;
    //first round
    for i in 0..core.graph.inmap.len() {
        let new_estimate = compute_index(core, i);
        if new_estimate < core.est[i] {
            core.est[i] = new_estimate;
            core.queues[0].push_back((i, new_estimate));
            core.changed[i] = true;
        }
    }
    let mut continua = true;
    while continua {
        continua = false;
        for i in 0..core.graph.inmap.len() {
            let new_estimate = compute_index(core, i);
            if new_estimate < core.est[i] {
                core.est[i] = new_estimate;
                core.queues[0].push_back((i, new_estimate));
                core.changed[i] = true;
                continua = true
            }
        }
    }

    //loop
    println!();
    for n in 0..core.count.len() {
        println!("{} : {}", n + 1, core.est[n])
    }
}

fn main() -> io::Result<()> {
    let file_path = "p2p-Gnutella08.txt";

    let mut graph = Graph::new();

    // Open the file
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    // Iterate over each line in the file
    for line in reader.lines() {
        let line = line?; // Get the line, and propagate any errors
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

    graph.no_duplicates();
    graph.debug_print();

    let mut algorithm: Data = Data::new(graph);

    compute_coreness(&mut algorithm);

    Ok(())
}
