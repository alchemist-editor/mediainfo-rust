"""Print a DLL's exported names, one per line (for an import-library .def)."""
import struct
import sys

data = open(sys.argv[1], "rb").read()
pe = struct.unpack_from("<I", data, 0x3C)[0]
assert data[pe:pe + 4] == b"PE\0\0", "not a PE file"
sections_count = struct.unpack_from("<H", data, pe + 6)[0]
optional_size = struct.unpack_from("<H", data, pe + 20)[0]
optional = pe + 24
assert struct.unpack_from("<H", data, optional)[0] == 0x20B, "not PE32+"
export_rva = struct.unpack_from("<I", data, optional + 112)[0]
sections = []
for i in range(sections_count):
    at = optional + optional_size + 40 * i
    vsize, va, _, raw = struct.unpack_from("<IIII", data, at + 8)
    sections.append((va, max(vsize, 1), raw))


def offset(rva):
    for va, size, raw in sections:
        if va <= rva < va + size:
            return rva - va + raw
    raise ValueError(hex(rva))


directory = offset(export_rva)
count, names_rva = struct.unpack_from("<I", data, directory + 24)[0], struct.unpack_from("<I", data, directory + 32)[0]
for i in range(count):
    name_at = offset(struct.unpack_from("<I", data, offset(names_rva) + 4 * i)[0])
    print(data[name_at:data.index(b"\0", name_at)].decode())
