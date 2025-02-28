use std::cmp;
use std::cmp::max;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, Write};

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

    pub fn debug_print(&mut self) {
        println!("Number of nodes: {}", self.inmap.len());
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

struct Data {
    graph: Graph,
    est: Vec<usize>,
    changed: Vec<bool>,
    count: Vec<usize>,
    queue: VecDeque<usize>, //all'iterazione i uso i%2 come coda attuale e i+1%2 come coda per la prossima iterazione
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
    for i in 0..core.graph.inmap.len() {
        core.queue.push_front(i);
    }

    while !core.queue.is_empty() {
        if let Some(node) = core.queue.pop_front() {
            core.changed[node] = false;
            let old_estimate = core.est[node];
            let new_estimate = compute_index(core, node);
            if new_estimate < old_estimate {
                for i in &core.graph.inmap[node] {
                    if !core.changed[*i]
                        && new_estimate < core.est[*i]
                        && old_estimate >= core.est[*i]
                    {
                        core.queue.push_front(*i);
                        core.changed[*i] = true;
                    }
                }
                core.est[node] = new_estimate;
            }
        }
    }

    //loop
    println!();
    let lim = cmp::min(30, core.est.len());
    for n in 0..lim {
        println!("{} : {}", n, core.est[n])
    }
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

    //loop
    println!();
    let lim = cmp::min(30, core.est.len());
    for n in 0..lim {
        println!("{} : {}", n, core.est[n]);
    }
}

fn write_to_file(vec: Vec<usize>, filename: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(filename)?;

    for element in vec {
        writeln!(file, "{}", element)?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let file_path = "web-Stanford.txt";

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

    compute_coreness_queue(&mut algorithm);

    let _ = write_to_file(algorithm.est, "./tests/web-Stanford_core.txt");

    Ok(())
}
