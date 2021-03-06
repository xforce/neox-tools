import types
try:
    import cStringIO
except:
    from io import BytesIO
import sys

PYTHON3 = sys.version_info >= (3, 0)


def try_ord(x):
    if type(x) is not int:
        return ord(x)
    else:
        return x


class Unicode(object):
    def __init__(self, val):
        self.value = val


class BinaryComplex(object):
    def __init__(self, real, imag):
        self.real = real
        self.imag = imag


class StringRef(object):
    def __init__(self, val, index):
        self.value = val
        self.index = index

    def decode(self):
        return self.value.decode()


class Intern(object):
    def __init__(self, val):
        self.value = val

    def decode(self):
        return self.value.decode()

# UNUSED!


class Int64(object):
    def __init__(self, val):
        self.value = val


class CodeType(object):
    def __init__(self, argcount, nlocals, stacksize, flags, code, consts,
                 names, varnames, filename, name, firstlineno,
                 lnotab, freevars, cellvars):
        self.orig_args = (argcount, nlocals, stacksize, flags, code, consts,
                          names, varnames, filename, name, firstlineno,
                          lnotab, freevars, cellvars)
        if PYTHON3:
            self.co_filename = filename.decode()
            self.code = types.CodeType(argcount, 0, 0, nlocals, stacksize, flags, code, consts,
                                       tuple(map(lambda x: x.decode(), names)), tuple(map(lambda x: x.decode(), varnames)
                                                                                      ), filename.decode(
                                       ), name.decode(), firstlineno,
                                       lnotab, tuple(map(lambda x: x.decode(), freevars)
                                                     ), tuple(map(lambda x: x.decode(), cellvars)
                                                              ))
        else:
            self.co_filename = filename
            self.code = types.CodeType(argcount, nlocals, stacksize, flags, code, consts,
                                       names, varnames, filename, name, firstlineno,
                                       lnotab, freevars, cellvars)


TYPE_NULL = '0'
TYPE_NONE = 'N'
TYPE_FALSE = 'F'
TYPE_TRUE = 'T'
TYPE_STOPITER = 'S'
TYPE_ELLIPSIS = '.'
TYPE_INT = 'i'
TYPE_INT64 = 'I'
TYPE_FLOAT = 'f'
TYPE_BINARY_FLOAT = 'g'
TYPE_COMPLEX = 'x'
TYPE_BINARY_COMPLEX = 'y'
TYPE_LONG = 'l'
TYPE_STRING = 's'
TYPE_INTERNED = 't'
TYPE_STRINGREF = 'R'
TYPE_TUPLE = '('
TYPE_LIST = '['
TYPE_DICT = '{'
TYPE_CODE = 'c'
TYPE_UNICODE = 'u'
TYPE_UNKNOWN = '?'
TYPE_SET = '<'
TYPE_FROZENSET = '>'

UNKNOWN_BYTECODE = 0


class _NULL:
    pass


class _Marshaller:
    dispatch = {}

    def __init__(self, writefunc, opmap=None):
        self._write = writefunc
        self._opmap = opmap or {}
        self._stringtable = []

    def dump(self, x):
        try:
            self.dispatch[type(x)](self, x)
        except KeyError:
            for tp in type(x).mro():
                func = self.dispatch.get(tp)
                if func:
                    break
            else:
                raise ValueError("unmarshallable object")
            func(self, x)

    def w_long64(self, x):
        self.w_long(x)
        self.w_long(x >> 32)

    def w_long(self, x):
        a = chr(x & 0xff)
        x >>= 8
        b = chr(x & 0xff)
        x >>= 8
        c = chr(x & 0xff)
        x >>= 8
        d = chr(x & 0xff)
        self._write(a + b + c + d)

    def w_short(self, x):
        self._write(chr((x) & 0xff))
        self._write(chr((x >> 8) & 0xff))

    def dump_none(self, x):
        self._write(TYPE_NONE)

    dispatch[type(None)] = dump_none

    def dump_bool(self, x):
        if x:
            self._write(TYPE_TRUE)
        else:
            self._write(TYPE_FALSE)

    dispatch[bool] = dump_bool

    def dump_stopiter(self, x):
        if x is not StopIteration:
            raise ValueError("unmarshallable object")
        self._write(TYPE_STOPITER)

    dispatch[type(StopIteration)] = dump_stopiter

    def dump_ellipsis(self, x):
        self._write(TYPE_ELLIPSIS)

    try:
        dispatch[type(Ellipsis)] = dump_ellipsis
    except NameError:
        pass

    # In Python3, this function is not used; see dump_long() below.
    def dump_int(self, x):
        y = x >> 31
        if y and y != -1:
            self._write(TYPE_INT64)
            self.w_long64(x)
        else:
            self._write(TYPE_INT)
            self.w_long(x)

    dispatch[int] = dump_int

    def dump_long(self, x):
        self._write(TYPE_LONG)
        sign = 1
        if x < 0:
            sign = -1
            x = -x
        digits = []
        while x:
            digits.append(x & 0x7FFF)
            x = x >> 15
        self.w_long(len(digits) * sign)
        for d in digits:
            self.w_short(d)

    try:
        long
    except NameError:
        # We want to produce Python2 output
        # This would cause all ints to be dumped as longs
        # Which is not what we want...
        if not PYTHON3:
            dispatch[int] = dump_long
    else:
        dispatch[long] = dump_long

    def dump_float(self, x):
        write = self._write
        write(TYPE_FLOAT)
        s = repr(x)
        write(chr(len(s)))
        write(s)

    dispatch[float] = dump_float

    def dump_binary_float(self, x):
        write = self._write
        write(TYPE_BINARY_FLOAT)
        import struct
        write(struct.pack('d', x))

    def dump_complex(self, x):
        write = self._write
        write(TYPE_COMPLEX)
        s = repr(x.real)
        write(chr(len(s)))
        write(s)
        s = repr(x.imag)
        write(chr(len(s)))
        write(s)

    try:
        dispatch[complex] = dump_complex
    except NameError:
        pass

    def dump_binary_complex(self, x):
        import struct
        write = self._write
        write(TYPE_BINARY_COMPLEX)
        write(struct.pack('d', x.real))
        write(struct.pack('d', x.imag))

    dispatch[BinaryComplex] = dump_binary_complex

    def dump_string(self, x):
        # XXX we can't check for interned strings, yet,
        # so we (for now) never create TYPE_INTERNED or TYPE_STRINGREF
        self._write(TYPE_STRING)
        self.w_long(len(x))
        self._write(x)

    dispatch[bytes] = dump_string

    def dump_interned(self, x):
        self._stringtable.append(x.value)
        self._write(TYPE_INTERNED)
        self.w_long(len(x.value))
        self._write(x.value)

    dispatch[Intern] = dump_interned

    def dump_stringref(self, x):
        self._write(TYPE_STRINGREF)
        self.w_long(self._stringtable.index(x.value))

    dispatch[StringRef] = dump_stringref

    def dump_unicode(self, x):
        # TODO(alexander):
        # There are cases in Python 2 where this _should_
        # end up being string to make uncompyle work
        # I think that is a limitation in uncompyle, but OH WELL
        # We _can_ detect this when this script is run via Python 3
        # but not when run via Python 2...which is somewhat unfortunate
        is_unicode = type(x) is Unicode
        is_bytes = is_unicode and type(x.value) is bytes
        if (is_unicode and not is_bytes) or not PYTHON3:
            self._write(TYPE_UNICODE)
        else:
            self._write(TYPE_STRING)

        if is_unicode:
            s = x.value
            if not is_bytes:
                s = x.value.encode('utf8')
        else:
            s = x.encode('utf8')
        self.w_long(len(s))
        self._write(s)

    try:
        unicode
    except NameError:
        dispatch[str] = dump_unicode
        dispatch[Unicode] = dump_unicode
    else:
        dispatch[unicode] = dump_unicode

    def dump_tuple(self, x):
        self._write(TYPE_TUPLE)
        self.w_long(len(x))
        for item in x:
            self.dump(item)

    dispatch[tuple] = dump_tuple

    def dump_list(self, x):
        self._write(TYPE_LIST)
        self.w_long(len(x))
        for item in x:
            self.dump(item)

    dispatch[list] = dump_list

    def dump_dict(self, x):
        self._write(TYPE_DICT)
        for key, value in x.items():
            self.dump(key)
            self.dump(value)
        self._write(TYPE_NULL)

    dispatch[dict] = dump_dict

    def dump_code(self, x):
        (co_argcount, co_nlocals, co_stacksize, co_flags, _code, co_consts,
         co_names, co_varnames, co_filename, co_name, co_firstlineno,
         co_lnotab, co_freevars, co_cellvars) = x.orig_args
        self._write(TYPE_CODE)
        self.w_long(co_argcount)
        self.w_long(co_nlocals)
        self.w_long(co_stacksize)
        self.w_long(co_flags)

        self.dump(self._transform_opcode(x.code.co_code))

        self.dump(co_consts)
        self.dump(co_names)
        self.dump(co_varnames)
        self.dump(co_freevars)
        self.dump(co_cellvars)
        self.dump(co_filename)
        self.dump(co_name)
        self.w_long(co_firstlineno)
        self.dump(co_lnotab)

    try:
        dispatch[CodeType] = dump_code
    except NameError:
        pass

    def _transform_opcode(self, x):
        if not self._opmap:
            return x

        opcode = bytearray(x)
        c = 0
        while c < len(opcode):
            try:
                n = self._opmap[opcode[c]]
            except Exception as e:
                print("unmapping %s" % opcode[c])
                print(e)

            opcode[c] = n

            if n < 90:
                c += 1
            else:
                c += 3
        if not PYTHON3:
            return str(opcode)
        else:
            return bytes(opcode)

    def dump_set(self, x):
        self._write(TYPE_SET)
        self.w_long(len(x))
        for each in x:
            self.dump(each)

    try:
        dispatch[set] = dump_set
    except NameError:
        pass

    def dump_frozenset(self, x):
        self._write(TYPE_FROZENSET)
        self.w_long(len(x))
        for each in x:
            self.dump(each)

    try:
        dispatch[frozenset] = dump_frozenset
    except NameError:
        pass


class _Unmarshaller:
    dispatch = {}

    def __init__(self, readfunc):
        self._read = readfunc
        self._stringtable = []

    def load(self):
        c = self._read(1)
        if not c:
            raise EOFError
        try:
            if type(c) is not str:
                c = c.decode()
            return self.dispatch[c](self)
        except KeyError:
            print(self.dispatch.keys())
            raise ValueError("bad marshal code: %c (%d)" % (c, try_ord(c)))

    def r_short(self):
        lo = try_ord(self._read(1))
        hi = try_ord(self._read(1))
        x = lo | (hi << 8)
        if x & 0x8000:
            x = x - 0x10000
        return x

    def r_long(self):
        s = self._read(4)
        a = try_ord(s[0])
        b = try_ord(s[1])
        c = try_ord(s[2])
        d = try_ord(s[3])
        x = a | (b << 8) | (c << 16) | (d << 24)
        if d & 0x80 and x > 0:
            x = -((1 << 32) - x)
            return int(x)
        else:
            return x

    def r_long64(self):
        a = try_ord(self._read(1))
        b = try_ord(self._read(1))
        c = try_ord(self._read(1))
        d = try_ord(self._read(1))
        e = try_ord(self._read(1))
        f = try_ord(self._read(1))
        g = try_ord(self._read(1))
        h = try_ord(self._read(1))
        x = a | (b << 8) | (c << 16) | (d << 24)
        x = x | (e << 32) | (f << 40) | (g << 48) | (h << 56)
        if h & 0x80 and x > 0:
            x = -((1 << 64) - x)
        return x

    def load_null(self):
        return _NULL

    dispatch[TYPE_NULL] = load_null

    def load_none(self):
        return None

    dispatch[TYPE_NONE] = load_none

    def load_true(self):
        return True

    dispatch[TYPE_TRUE] = load_true

    def load_false(self):
        return False

    dispatch[TYPE_FALSE] = load_false

    def load_stopiter(self):
        return StopIteration

    dispatch[TYPE_STOPITER] = load_stopiter

    def load_ellipsis(self):
        return Ellipsis

    dispatch[TYPE_ELLIPSIS] = load_ellipsis

    dispatch[TYPE_INT] = r_long

    dispatch[TYPE_INT64] = r_long64

    def load_long(self):
        size = self.r_long()
        sign = 1
        if size < 0:
            sign = -1
            size = -size
        x = 0
        for i in range(size):
            d = self.r_short()
            x = x | (d << (i * 15))
        return x * sign

    dispatch[TYPE_LONG] = load_long

    def load_float(self):
        n = try_ord(self._read(1))
        s = self._read(n)
        return float(s)

    dispatch[TYPE_FLOAT] = load_float

    def load_binary_float(self):
        import struct
        return struct.unpack('d', self._read(8))[0]

    dispatch[TYPE_BINARY_FLOAT] = load_binary_float

    def load_complex(self):
        n = try_ord(self._read(1))
        s = self._read(n)
        real = float(s)
        n = try_ord(self._read(1))
        s = self._read(n)
        imag = float(s)
        return complex(real, imag)

    dispatch[TYPE_COMPLEX] = load_complex

    def load_binary_complex(self):
        real = self.load_binary_float()
        imag = self.load_binary_float()
        return BinaryComplex(real, imag)

    dispatch[TYPE_BINARY_COMPLEX] = load_binary_complex

    def load_string(self):
        n = self.r_long()
        return self._read(n)

    dispatch[TYPE_STRING] = load_string

    def load_interned(self):
        n = self.r_long()
        ret = self._read(n)
        self._stringtable.append(ret)
        return Intern(ret)

    dispatch[TYPE_INTERNED] = load_interned

    def load_stringref(self):
        n = self.r_long()
        return StringRef(self._stringtable[n], n)

    dispatch[TYPE_STRINGREF] = load_stringref

    def load_unicode(self):
        n = self.r_long()
        s = self._read(n)
        # if decoding to uf8 failes
        # then this is likely a bytes (str in python2) thing
        # _should_ be fine
        try:
            ret = s.decode('utf8')
        except:
            ret = s
        if PYTHON3:
            ret = Unicode(ret)
        return ret

    dispatch[TYPE_UNICODE] = load_unicode

    def load_tuple(self):
        return tuple(self.load_list())

    dispatch[TYPE_TUPLE] = load_tuple

    def load_list(self):
        n = self.r_long()
        list = [self.load() for i in range(n)]
        return list

    dispatch[TYPE_LIST] = load_list

    def load_dict(self):
        d = {}
        while 1:
            key = self.load()
            if key is _NULL:
                break
            value = self.load()
            d[key] = value
        return d

    dispatch[TYPE_DICT] = load_dict

    def load_code(self):
        argcount = self.r_long()
        nlocals = self.r_long()
        stacksize = self.r_long()
        flags = self.r_long()
        code = self.load()
        consts = self.load()
        names = self.load()
        varnames = self.load()
        freevars = self.load()
        cellvars = self.load()
        filename = self.load()
        name = self.load()
        firstlineno = self.r_long()
        lnotab = self.load()
        r = CodeType(argcount, nlocals, stacksize, flags, code, consts,
                     names, varnames, filename, name, firstlineno,
                     lnotab, freevars, cellvars)
        return r

    dispatch[TYPE_CODE] = load_code

    def load_set(self):
        n = self.r_long()
        args = [self.load() for i in range(n)]
        return set(args)

    dispatch[TYPE_SET] = load_set

    def load_frozenset(self):
        n = self.r_long()
        args = [self.load() for i in range(n)]
        return frozenset(args)

    dispatch[TYPE_FROZENSET] = load_frozenset


def dump(x, f, opmap=None):
    if not PYTHON3:
        writefunc = f.write
    else:
        def writefunc(x):
            if type(x) is not bytes:
                x = bytearray(map(lambda x: int(try_ord(x)), x))
                return f.write(x)
            else:
                return f.write(x)
    m = _Marshaller(writefunc, opmap)
    m.dump(x)


def load(f):
    um = _Unmarshaller(f.read)
    return um.load()


def loads(content):
    if not PYTHON3:
        io = cStringIO.StringIO(content)
    else:
        io = BytesIO(content)
    return load(io)


def dumps(x, opmap=None):
    if not PYTHON3:
        io = cStringIO.StringIO()
    else:
        io = BytesIO()
    dump(x, io, opmap)
    io.seek(0)
    return io.read()
