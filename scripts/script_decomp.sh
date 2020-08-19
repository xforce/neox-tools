#!/bin/bash

mkdir -p script/temp
mkdir -p script/pyc
mkdir -p script/out

for file in "script_nxs"/*.nxs
do
    file="$(basename "$file")"
    echo $file
    python2 scripts/script_redirect.py script_nxs/$file > script/temp/$file.out
    python2 scripts/pyc_decryptor.py script/temp/$file.out script/pyc/$file.pyc
    python3 scripts/decompile_pyc.py -o script/out/$file.py script/pyc/$file.pyc 2> /dev/null
    if [ $? -ne 0 ]
    then 
        echo "Failed...sad face"
        # echo "Trying pycdc"
        # pycdc/pycdc script/pyc/$file.pyc > script/out/$file.py
        # if [ $? -ne 0 ]
        # then
        #     python3 tools/decompile_pyc.py -o script/out/$file.py script/pyc/$file.pyc 2> /dev/null
        # fi
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
done
