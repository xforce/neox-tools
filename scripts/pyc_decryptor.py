#! /usr/bin/env python
# -*- coding: utf-8 -*-
import os
import zlib
import marshal
import binascii
import argparse
import sys
import pymarshal

PYTHON3 = sys.version_info >= (3, 0)


class PYCEncryptor(object):
    def __init__(self):
        # LEFT (ORIGNAL) RIGHT (NEOX)
        self.opcode_encrypt_map = {
            0: 0,   # STOP CODE (TODO)
            1: 38,  # POP TOP
            2: 46,  # ROT TWO
            3: 37,  # ROT THREE
            4: 66,  # DUP TOP
            5: 12,  # ROT FOUR
            # 9: 13,   # NOP (TODO)
            10: 35,  # UNARY POSITIVE
            11: 67,  # UNARY NEGATIVE
            12: 81,  # UNARY_NOT
            13: 32,  # UNARY_CONVERT
            15: 9,  # UNARY_INVERT
            19: 63,  # BINARY_POWER
            20: 70,  # BINARY_MULTIPLY
            21: 44,  # BINARY_DIVIDE
            22: 36,  # BINARY_MODULO
            23: 39,  # BINARY_ADD
            24: 57,  # BINARY_SUBTRACT
            25: 10,  # BINARY_SUBSCR
            26: 52,  # BINARY_FLOOR_DIVIDE
            27: 13,  # BINARY_TRUE_DIVIDE (TODO)
            28: 49,  # INPLACE_FLOOR_DIVIDE
            # 29: 29, # INPLACE_TRUE_DIVIDE (TODO)
            30: 86,  # SLICE
            31: 87,  # SLICE_1
            32: 88,  # SLICE_2
            33: 89,  # SLICE_3
            40: 24,  # STORE_SLICE
            41: 25,  # STORE_SLICE_1
            42: 26,  # STORE_SLICE_2
            43: 27,  # STORE_SLICE_3
            50: 14,  # DELETE_SLICE
            51: 15,  # DELETE_SLICE_1
            52: 16,  # DELETE_SLICE_2
            53: 17,  # DELETE_SLICE_3
            54: 8,  # STORE_MAP
            55: 21,  # INPLACE_ADD
            56: 55,  # INPLACE_SUBTRACT
            57: 82,  # INPLACE_MULTIPLY
            58: 34,  # INPLACE_DIVIDE
            59: 22,  # INPLACE_MODULO
            60: 65,  # STORE_SUBSCR
            61: 6,  # DELETE_SUBSCR
            62: 58,  # BINARY_LSHIFT
            63: 71,  # BINARY_RSHIFT
            64: 43,  # BINARY_AND
            65: 30,  # BINARY_XOR
            66: 19,  # BINARY_OR
            67: 5,  # INPLACE_POWER
            68: 60,  # GET_ITER
            # 70: 75,  # PRINT_EXPR (TODO, WIP)
            71: 53,  # PRINT_ITEM
            72: 42,  # PRINT_NEWLINE
            73: 3,  # PRINT_ITEM_TO
            74: 48,  # PRINT_NEWLINE_TO
            75: 84,  # INPLACE_LSHIFT
            76: 77,  # INPLACE_RSHIFT
            77: 78,  # INPLACE_AND
            78: 85,  # INPLACE_XOR
            79: 47,  # INPLACE_OR
            80: 51,  # BREAK_LOOP
            81: 54,  # WITH_CLEANUP
            82: 50,  # LOAD_LOCALS
            83: 83,  # RETURN_VALUE
            84: 74,  # IMPORT_STAR
            85: 64,  # EXEC_STMT
            86: 31,  # YIELD_VALUE
            87: 72,  # POP_BLOCK
            88: 45,  # END_FINALLY
            89: 33,  # BUILD_CLASS
            90: 145,    # HAVE_ARGUMENT/ STORE_NAME
            91: 159,    # DELETE_NAME
            92: 125,    # UNPACK_SEQUENCE
            93: 149,    # FOR_ITER
            94: 157,    # LIST_APPEND
            95: 132,    # STORE_ATTR
            96: 95,     # DELETE_ATTR
            97: 113,    # STORE_GLOBAL
            98: 111,    # DELETE_GLOBAL
            99: 138,    # DUP_TOPX
            100: 153,   # LOAD_CONST
            101: 101,   # LOAD_NAME
            102: 135,   # BUILD_TUPLE
            103: 90,    # BUILD_LIST
            104: 99,    # BUILD_SET
            105: 151,   # BUILD_MAP
            106: 96,    # LOAD_ATTR
            107: 114,   # COMPARE_OP
            108: 134,   # IMPORT_NAME
            109: 116,   # IMPORT_FROM
            110: 156,   # JUMP_FORWARD
            111: 105,   # JUMP_IF_FALSE_OR_POP
            112: 130,   # JUMP_IF_TRUE_OR_POP
            113: 137,   # JUMP_ABSOLUTE
            114: 148,   # POP_JUMP_IF_FALSE
            115: 172,   # POP_JUMP_IF_TRUE
            116: 155,   # LOAD_GLOBAL
            119: 103,   # CONTINUE_LOOP
            120: 158,   # SETUP_LOOP
            121: 128,   # SETUP_EXCEPT
            122: 110,   # SETUP_FINALLY
            124: 97,    # LOAD_FAST
            125: 104,   # STORE_FAST
            126: 118,   # DELETE_FAST
            130: 93,    # RAISE_VARARGS
            131: 131,   # CALL_FUNCTION
            132: 136,   # MAKE_FUNCTION
            133: 115,   # BUILD_SLICE
            134: 100,   # MAKE_CLOSURE
            135: 120,   # LOAD_CLOSURE
            136: 129,   # LOAD_DEREF
            137: 102,   # STORE_DEREF
            140: 140,   # CALL_FUNCTION_VAR
            141: 141,   # CALL_FUNCTION_KW
            142: 142,   # CALL_FUNCTION_VAR_KW
            143: 94,    # SETUP_WITH
            # SPECIAL NEOX THING, LOAD CONST + LOAD FAST, I think (TODO, GUESS)
            173: 173,
            146: 109,   # SET_ADD
            147: 123    # MAP_ADD
        }
        self.opcode_decrypt_map = {
            self.opcode_encrypt_map[key]: key for key in self.opcode_encrypt_map}
        self.pyc27_header = "\x03\xf3\x0d\x0a\x00\x00\x00\x00"

    def _decrypt_file(self, filename):
        os.path.splitext(filename)
        content = open(filename, "rb").read()

        try:
            m = pymarshal.loads(content)
        except RuntimeError as e:
            print(e)
            try:
                m = marshal.loads(content)
            except Exception as e:
                print("[!] error: %s" % str(e))
                return None
        return m.co_filename.replace('\\', '/'), pymarshal.dumps(m, self.opcode_decrypt_map)

    def decrypt_file(self, input_file, output_file=None):
        result = self._decrypt_file(input_file)
        if not result:
            return
        pyc_filename, pyc_content = result
        if not output_file:
            output_file = os.path.basename(pyc_filename) + '.pyc'
        with open(output_file, 'wb') as fd:
            if not PYTHON3:
                fd.write(self.pyc27_header + pyc_content)
            else:
                fd.write(bytearray(
                    map(lambda x: int(ord(x)), self.pyc27_header)) + pyc_content)


def main():
    parser = argparse.ArgumentParser(description='neox py decrypt tool')
    parser.add_argument("INPUT_NAME", help='input file')
    parser.add_argument("OUTPUT_NAME", help='output file')
    args = parser.parse_args()
    encryptor = PYCEncryptor()
    encryptor.decrypt_file(args.INPUT_NAME, args.OUTPUT_NAME)


if __name__ == '__main__':
    main()
