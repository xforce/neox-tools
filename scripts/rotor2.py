import ctypes
import math
import sys

PYTHON3 = sys.version_info >= (3, 0)


def to_int(x, d=0):
    if not PYTHON3:
        return int(math.trunc(x))
    else:
        return int(x)


class Rotor(object):
    def __init__(self, key):
        self.key = [0, 0, 0, 0, 0]
        self.seed = [0, 0, 0]
        self.size = 256
        self.rotors = 6
        self.e_rotor = [[0] * self.size] * (self.rotors)
        self.d_rotor = [[0] * self.size] * (self.rotors)
        self._positions = [0] * (self.rotors)
        self._advances = [0] * (self.rotors)

        self.set_key(key)
        self.set_seed()
        self.positions()
        self.advances()
        self.e_rotors()
        self.d_rotors()

        for i in range(self.rotors):
            self._positions[i] = ctypes.c_uint8(self.r_rand(
                ctypes.c_short(self.size).value)).value
            self._advances[i] = (
                1+(2*(self.r_rand(ctypes.c_short(int(self.size/2.0)).value))))
            (e_rotor, d_rotor) = self.RTR_permute_rotor(
                self.e_rotor[i][:], self.d_rotor[i][:])
            self.e_rotor[i] = e_rotor
            self.d_rotor[i] = d_rotor

    def RTR_make_id_rotor(self, rtr):
        for j in range(self.size):
            rtr[j] = ctypes.c_uint8(j).value
        return rtr

    def RTR_permute_rotor(self, e, d):
        i = self.size
        self.RTR_make_id_rotor(e)
        while 2 <= i:
            q = self.r_rand(i)
            i -= 1
            j = e[q]
            e[q] = ctypes.c_uint8(e[i]).value
            e[i] = ctypes.c_uint8(j).value
            d[j] = ctypes.c_uint8(i).value

        e[0] = ctypes.c_uint8(e[0]).value
        d[e[0]] = 0

        return (e, d)

    def d_rotors(self):
        for i in range(self.rotors):
            for j in range(self.size):
                self.d_rotor[i][j] = ctypes.c_uint8(j).value
        pass

    def e_rotors(self):
        for i in range(self.rotors):
            self.e_rotor[i] = self.RTR_make_id_rotor(self.e_rotor[i])

    def positions(self):
        for i in range(self.rotors):
            self._positions[i] = 1

    def advances(self):
        for i in range(self.rotors):
            self._advances[i] = 1

    def set_seed(self):
        self.seed[0] = self.key[0]
        self.seed[1] = self.key[1]
        self.seed[2] = self.key[2]

    def set_key(self, key):
        k1 = 995
        k2 = 576
        k3 = 767
        k4 = 671
        k5 = 463

        for ki in key:
            ki = ord(ki)
            k1 = (((k1 << 3 | k1 >> 13) + ki) & 0xFFFF)
            k2 = (((k2 << 3 | k2 >> 13) ^ ki) & 0xFFFF)
            k3 = (((k3 << 3 | k3 >> 13) - ki) & 0xFFFF)
            k4 = ((ki - (k4 << 3 | k4 >> 13)) & 0xFFFF)
            k5 = (((k5 << 3 | k5 >> 13) ^ (~ki & 0xFFFF)) & 65535)

        self.key[0] = ctypes.c_short(k1).value
        self.key[1] = ctypes.c_short(k2 | 1).value
        self.key[2] = ctypes.c_short(k3).value
        self.key[3] = ctypes.c_short(k4).value
        self.key[4] = ctypes.c_short(k5).value

    def RTR_advance(self):
        i = 0
        temp = 0
        while (i < self.rotors):
            temp = self._positions[i] + self._advances[i]
            self._positions[i] = ctypes.c_uint8(
                to_int(math.fmod(temp, self.size))).value

            if (temp >= self.size) and (i < (self.rotors - 1)):
                self._positions[i+1] = ctypes.c_uint8(1 +
                                                      self._positions[i+1]).value

            i += 1

    def RTR_d_char(self, c):
        i = self.rotors - 1
        tc = c

        while 0 <= i:
            # print(i, tc, self.d_rotor[i][tc])
            tc = ctypes.c_uint8((self._positions[i] ^
                                 to_int(math.fmod(self.d_rotor[i][tc], self.size)))).value
            i -= 1
        self.RTR_advance()
        return tc

    def decrypt(self, data):
        data_put = bytearray()
        for i in range(len(data)):
            d = data[i]
            if type(d) is str:
                d = ord(d)
            # print(self.RTR_d_char(d))
            # return
            data_put.append(self.RTR_d_char(d))
        return bytes(data_put)

    def r_random(self):
        x = self.seed[0]
        y = self.seed[1]
        z = self.seed[2]

        # oy = y
        # ox = x
        x = 171 * to_int(math.fmod(x, 177)
                         ) - 2 * to_int(x/177.0)
        y = 172 * to_int(math.fmod(y, 176)
                         ) - 35 * to_int(y/176.0)
        z = 170 * to_int(math.fmod(z, 178)
                         ) - 63 * to_int(z/178.0)

        # print(x, ox, 171 * to_int(math.fmod(ox, 177.0)),
        #       int(ox/177.0),
        #       2 * to_int(ox/177.0))

        # asdjasdklajsldjalsjdlka

        # X: 25928, Y: -11943, Z: 4464

        if x < 0:
            x = x + 30269
        if y < 0:
            y = y + 30307
        if z < 0:
            z = z + 30323

        self.seed[0] = x
        self.seed[1] = y
        self.seed[2] = z

        term = (x/30269.0) + (y/30307.0) + (z/30323.0)
        val = term - to_int(term)

        if val >= 1.0:
            val = 0.0

        return val

    def r_rand(self, s):
        n = ctypes.c_short(int(self.r_random() * float(s))).value
        return ctypes.c_short(int(math.fmod(n, s))).value


def newrotor(key):
    return Rotor(key)
