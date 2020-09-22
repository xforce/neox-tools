#!/bin/bash

mkdir -p script/temp
mkdir -p script/pyc
mkdir -p script/out
mkdir -p script/failed

filecount=$(ls -1 script_nxs | wc -l)
counter=0

for file in script_nxs/*
do
    file="$(basename "$file")"
    echo "$((100*$counter/$filecount))% - $file"

    python2 scripts/script_redirect.py script_nxs/$file > script/temp/$file.out
    python2 scripts/pyc_decryptor.py script/temp/$file.out script/pyc/$file.pyc
    python3 scripts/decompile_pyc.py -o script/out/$file.py script/pyc/$file.pyc 2> /dev/null
    if [ $? -ne 0 ]; then 
        # A lot of the time Python 2 works instead
        python2 scripts/decompile_pyc.py -o script/out/$file.py script/pyc/$file.pyc 2> /dev/null
    
    fi

    if [ ! -f script/out/$file.py ]; then
        echo "Failed...sad face. Copied to 'script/failed/'"
        cp "script_nxs/$file" "script/failed"
        counter=$(($counter+1))
        continue
    fi

    file_name="$(head -n 5 script/out/$file.py | tail -n 1)"
    file_name=${file_name//\\/\/}
    regexp="# Embedded.*"
    if [[ "$file_name" =~ $regexp ]]; then
        file_name="$(echo "$file_name" | sed -e "s/# Embedded file name: //")"
        file_dir="$(dirname "$file_name")"
        echo $file_name
        echo $file_dir
        mkdir -p script/layout/$file_dir
        cp script/out/$file.py script/layout/$file_name
    fi
    counter=$(($counter+1))
done
