#!/usr/bin/env python2
import argparse
import sys
import os

dir_path = os.path.dirname(os.path.realpath(__file__))

sys.path.insert(0, os.path.join(dir_path, "."))
sys.path.insert(0, os.path.join(dir_path, "unnpk", "tools"))

from script_redirect import unnpk

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("INPUT_NAME", help='input file')
    args = parser.parse_args()
    input_file = args.INPUT_NAME
    data = unnpk(open(input_file).read())
    print(data)

if __name__ == '__main__':
    main()
