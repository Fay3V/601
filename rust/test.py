from cffi import FFI
import os
import sys

ffi = FFI()

# Read the C header file
header_file = "sm.h"
with open(header_file) as f:
    ffi.cdef(f.read())

# Load the shared library
libname = "libsm.so"
libpath = os.path.join(os.path.dirname(__file__), "target","x86_64-unknown-linux-gnu", "release", libname)

lib = ffi.dlopen(libpath)

class StateMachine(object):
    def __init__(self, cobj):
        self._c = cobj

    @staticmethod
    def new():
        cobj = lib.sm_new()
        return StateMachine(cobj)

    def step(self, inp):
        return lib.sm_step(self._c, inp)

b = StateMachine.new()
for i in range(0, 100):
    print b.step(0)
