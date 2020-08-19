#!/usr/bin/env python2
import zlib
import argparse

def _reverse_string(s):
    l = list(s)
    l = map(lambda x: chr(ord(x) ^ 154), l[0:128]) + l[128:]
    l.reverse()
    return ''.join(l)


def unpack_pyc(data):
    asdf_dn = 'j2h56ogodh3se'
    asdf_dt = '=dziaq.'
    asdf_df = '|os=5v7!"-234'
    asdf_tm = asdf_dn * 4 + (asdf_dt + asdf_dn + asdf_df) * 5 + '!' + '#' + asdf_dt * 7 + asdf_df * 2 + '*' + '&' + "'"
    import rotor
    rotor = rotor.newrotor(asdf_tm)
    data = rotor.decrypt(data)
    data = zlib.decompress(data)
    data = _reverse_string(data)
    return data

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("INPUT_NAME", help='input file')
    args = parser.parse_args()
    input_file = args.INPUT_NAME
    data = unpack_pyc(open(input_file).read())
    print(data)

if __name__ == '__main__':
    main()
