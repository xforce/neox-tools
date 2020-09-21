#!/usr/bin/env python
import argparse
import sys
import os

dir_path = os.path.dirname(os.path.realpath(__file__))

sys.path.insert(1, dir_path)
sys.path.insert(0, os.path.join(dir_path, "unnpk", "tools"))

def try_ord(x):
    if type(x) is not int:
        return ord(x)
    else:
        return x

# Python 3 compatible version of the function found in unnpk
def rotate_string_py3(s):
    l = list(s)
    l = list(map(lambda x: chr(try_ord(x) ^ 154), l[0:128])) + l[128:]
    l.reverse()
    return bytearray(map(lambda x: int(try_ord(x)), l))

import script_redirect

script_redirect._reverse_string = rotate_string_py3
script_redirect.ord = try_ord

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('INPUT_NAME', help='input file')
    parser.add_argument('output_name', nargs='?',
                        help='output file', default=None)
    args = parser.parse_args()
    input_file = args.INPUT_NAME
    data = script_redirect.unnpk(open(input_file, "rb").read())
    output_file = args.output_name
    if output_file is None:
        os.write(sys.stdout.fileno(), data)
    else:
        with open(output_file, "wb") as f:
            f.write(data)

if __name__ == '__main__':
    main()
