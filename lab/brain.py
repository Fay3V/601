from cffi import FFI
import os
import sys
# import math
from soar.io import io
# from lib601 import gfx, util
# import lib601.sm as sm

ffi = FFI()

# Read the C header file
header_file = "/course/lab/sm.h"
with open(header_file) as f:
    ffi.cdef(f.read())

# Load the shared library
libname = "/course/lab/libsm.so"
lib = ffi.dlopen(libname)

class Brain(object):
    def __init__(self, cobj):
        self._c = cobj

    def start(self):
        lib.sm_reset(self._c)

    def isDone(self):
        return lib.sm_is_done(self._c)

    def step(self, sensor_input):
        # print inp
        input = ffi.new("SensorInput_t *")

        for i in range(8):
            input.sonars.idx[i] = sensor_input.sonars[i]        

        # Fill pose
        input.odometry.pos.x = sensor_input.odometry.x
        input.odometry.pos.y = sensor_input.odometry.y
        input.odometry.theta = sensor_input.odometry.theta

        output = lib.sm_step(self._c, input[0])
        action = io.Action(output.fvel, output.rvel)
        return action

    def run(self, n):
        val = lib.sm_run(self._c, n)
        return [val.ptr[i] for i in range(val.len)]
