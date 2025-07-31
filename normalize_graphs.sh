#!/bin/bash

# Script per normalizzare tutti i grafi contenuti nella directory graphs
# Ogni grafo è contenuto in una directory con il suo stesso nome
# L'output verrà salvato come "graph.txt" nella stessa directory

# Directory contenente i grafi
GRAPHS_DIR="./graphs"

# Controlla se la directory graphs esiste
if [ ! -d "$GRAPHS_DIR" ]; then
    echo "Errore: La directory $GRAPHS_DIR non esiste!"
    exit 1
fi

# Controlla se il normalizer esiste
if [ ! -f "./normalizer.py" ]; then
    echo "Errore: Il file normalizer.py non esiste!"
    exit 1
fi

echo "Inizio normalizzazione dei grafi..."
echo "=================================="

# Itera attraverso tutte le directory in graphs
for graph_dir in "$GRAPHS_DIR"/*/; do
    # Controlla se è una directory valida
    if [ ! -d "$graph_dir" ]; then
        continue
    fi
    
    # Rimuove il trailing slash e ottiene solo il nome della directory
    dir_name=$(basename "$graph_dir")
    
    # Percorso del file di input (stesso nome della directory)
    input_file="$graph_dir$dir_name.txt"
    
    # Percorso del file di output
    output_file="$graph_dir/graph.txt"
    
    echo "Processando: $dir_name"
    
    # Controlla se il file di input esiste
    if [ ! -f "$input_file" ]; then
        echo "  ⚠️  File di input non trovato: $input_file"
        echo "  Cercando altri file nella directory..."
        
        # Lista tutti i file .txt nella directory
        txt_files=("$graph_dir"*.txt)
        if [ ${#txt_files[@]} -gt 0 ] && [ -f "${txt_files[0]}" ]; then
            # Prende il primo file .txt trovato
            input_file="${txt_files[0]}"
            echo "  ✓  Usando invece: $(basename "$input_file")"
        else
            echo "  ❌ Nessun file .txt trovato in $graph_dir"
            continue
        fi
    fi
    
    # Esegue il normalizer
    echo "  📊 Normalizzando $(basename "$input_file") -> graph.txt"
    
    if python3 ./normalizer.py "$input_file" "$output_file"; then
        echo "  ✅ Completato con successo!"
    else
        echo "  ❌ Errore durante la normalizzazione!"
    fi
    
    echo ""
done

echo "=================================="
echo "Normalizzazione completata!"

# Mostra un riepilogo dei file creati
echo ""
echo "File graph.txt creati:"
find "$GRAPHS_DIR" -name "graph.txt" -type f | sort
