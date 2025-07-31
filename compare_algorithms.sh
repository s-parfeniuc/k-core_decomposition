#!/bin/bash

# Script per eseguire 01.rs e 09.rs su tutti i grafi e confrontare i risultati

# Directory contenente i grafi
GRAPHS_DIR="./graphs"

# Controlla se la directory graphs esiste
if [ ! -d "$GRAPHS_DIR" ]; then
    echo "Errore: La directory $GRAPHS_DIR non esiste!"
    exit 1
fi

echo "Inizio esecuzione algoritmi..."
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
    input_file="$graph_dir/graph.txt"
    
    # Controlla se il file di input esiste
    if [ ! -f "$input_file" ]; then
        echo "  ⚠️  File di input non trovato: $input_file"
        continue
    fi
    
    echo "Processando: $dir_name"
    
    # File temporanei per l'output
    tmp_file_01="$(mktemp)"
    tmp_file_09="$(mktemp)"
    
    # Esegue gli algoritmi 01 e 09
    echo "  ▶️ Eseguendo algoritmo 01..."
    if ! ./target/release/01 "$input_file" "$tmp_file_01" 2>/dev/null; then
        echo "  ❌ Errore nell'esecuzione dell'algoritmo 01"
        rm "$tmp_file_01" "$tmp_file_09"
        continue
    fi

    echo "  ▶️ Eseguendo algoritmo 09..."
    if ! ./target/release/09 "$input_file" "$tmp_file_09" 2>/dev/null; then
        echo "  ❌ Errore nell'esecuzione dell'algoritmo 09"
        rm "$tmp_file_01" "$tmp_file_09"
        continue
    fi

    # Confronta l'output
    echo "  🔍 Confrontando i risultati..."
    if diff "$tmp_file_01" "$tmp_file_09" > /dev/null; then
        echo "  ✅ I risultati sono uguali!"
    else
        # Conta le righe diverse
        diff_lines=$(diff "$tmp_file_01" "$tmp_file_09" | grep -c "^[<>]")
        total_lines_01=$(wc -l < "$tmp_file_01")
        total_lines_09=$(wc -l < "$tmp_file_09")
        
        echo "  ❌ I risultati differiscono!"
        echo "     📊 Righe diverse: $diff_lines"
        echo "     📄 Righe algoritmo 01: $total_lines_01"
        echo "     📄 Righe algoritmo 09: $total_lines_09"
        
        # Mostra le prime 5 differenze per debug
        echo "     🔍 Prime 5 differenze:"
        diff "$tmp_file_01" "$tmp_file_09" | head -10 | sed 's/^/        /'
    fi

    # Rimuove i file temporanei
    rm "$tmp_file_01" "$tmp_file_09"
    
    echo ""
done

echo "=================================="
echo "Esecuzione completata!"
