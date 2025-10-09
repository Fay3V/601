from cffi import FFI
import os
import sys
from soar.io import io
from lib601 import gfx

ffi = FFI()

# Read the C header file
header_file = "sm.h"
with open(header_file) as f:
    ffi.cdef(f.read())

# Load the shared library
libname = "libsm.so"
lib = ffi.dlopen(libname)

class Brain(object):
    def __init__(self, cobj):
        self._c = cobj

    def step(self, sensor_input):
        # print inp
        input = ffi.new("SensorInput_t *")

        for i in range(8):
            input.sonars.idx[i] = sensor_input.sonars[i]        

        # Fill pose
        input.odometry.x = sensor_input.odometry.x
        input.odometry.y = sensor_input.odometry.y
        input.odometry.theta = sensor_input.odometry.theta

        output = lib.sm_step(self._c, input[0])
        action = io.Action(output.fvel, output.rvel)
        return action

    def run(self, n):
        val = lib.sm_run(self._c, n)
        return [val.ptr[i] for i in range(val.len)]

    def reset(self):
        return lib.sm_reset(self._c)


def setup():
    robot.behavior = Brain(lib.sm_simple())
    # robot.gfx = gfx.RobotGraphics(sonarMonitor = True, drawSlimeTrail = True)
    # startTheta = io.SensorInput().odometry.theta
    # robot.gfx.addStaticPlotFunction(
    #     x=('angle', lambda inp: startTheta-inp.odometry.theta),
    #     y=('left eye', lambda inp: inp.analogInputs[1]),
    #     connectPoints=True)    
    

def step():
    # action = robot.behavior.step(io.SensorInput())
    # print action
    robot.behavior.step(io.SensorInput()).execute()
