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

    def step(self, inp):
        return lib.sm_step(self._c, inp)

    def run(self, n):
        val = lib.sm_run(self._c, n)
        return [val.ptr[i] for i in range(val.len)]

    def reset(self):
        return lib.sm_reset(self._c)

# f = StateMachine(lib.sm_factorial())
# c = StateMachine(lib.sm_counter())
# for i in range(0, 100):
#     print "{0}! = {1}".format(c.step(1)-1, f.step(1))
 
w = StateMachine(lib.sm_world())
print w.run(30)
