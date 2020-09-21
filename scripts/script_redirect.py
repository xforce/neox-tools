#!/usr/bin/env python
import zlib
import argparse
import os
import sys


def try_ord(x):
    if type(x) is not int:
        return ord(x)
    else:
        return x


def _reverse_string(s):
    l = list(s)
    l = list(map(lambda x: chr(try_ord(x) ^ 154), l[0:128])) + l[128:]
    l.reverse()
    return bytearray(map(lambda x: int(try_ord(x)), l))


def unpack_pyc(data):
    import rotor2 as rotor
    key = "j2h56ogodh3sej2h56ogodh3sej2h56ogodh3sej2h56ogodh3se=dziaq.j2h56ogodh3se|os=5v7!\"-234=dziaq.j2h56ogodh3se|os=5v7!\"-234=dziaq.j2h56ogodh3se|os=5v7!\"-234=dziaq.j2h56ogodh3se|os=5v7!\"-234=dziaq.j2h56ogodh3se|os=5v7!\"-234!#=dziaq.=dziaq.=dziaq.=dziaq.=dziaq.=dziaq.=dziaq.|os=5v7!\"-234|os=5v7!\"-234*&'"
    rotor = rotor.newrotor(key)
    data = rotor.decrypt(data)
    data = zlib.decompress(data)
    data = _reverse_string(data)
    return data


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('INPUT_NAME', help='input file')
    parser.add_argument('output_name', nargs='?',
                        help='output file', default=None)
    args = parser.parse_args()
    input_file = args.INPUT_NAME
    data = unpack_pyc(open(input_file, "rb").read())

    output_file = args.output_name
    if output_file is None:
        os.write(sys.stdout.fileno(), data)
    else:
        with open(output_file, "wb") as f:
            f.write(data)


if __name__ == '__main__':
    main()
