from cffi import FFI
import os
import sys
import math
from soar.io import io
from lib601 import gfx, util
import lib601.sm as sm


ffi = FFI()

# Read the C header file
header_file = "sm.h"
with open(header_file) as f:
    ffi.cdef(f.read())

# Load the shared library
libname = "libsm.so"
lib = ffi.dlopen(libname)

class RotateTSM(sm.SM):
    rotationalGain = 3.0
    angleEpsilon = 0.01
    startState = 'start'

    def __init__(self, headingDelta):
        self.headingDelta = headingDelta

    def getNextValues(self, state, inp):
        currentTheta = inp.odometry.theta
        if state == 'start':
            thetaDesired = \
                util.fixAnglePlusMinusPi(currentTheta + self.headingDelta)
        else:
            (thetaDesired, thetaLast) = state

        newState = (thetaDesired, currentTheta)
        action = io.Action(rvel = self.rotationalGain * \
                            util.fixAnglePlusMinusPi(thetaDesired - currentTheta))
        return (newState, action)
    
    # def start(self):
    #     print "stttttttttttttttttttt"
    #     # super(RotateTSM, self).start()
    #     # return self.__super__.start()

    def done(self, state):
        if state == 'start':
            return False
        else:
            (thetaDesired, thetaLast) = state

        return util.nearAngle(thetaDesired, thetaLast, self.angleEpsilon)

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

def setup():
    robot.behavior = Brain(lib.sm_simple(0.3))
    robot.gfx = gfx.RobotGraphics(sonarMonitor = False, drawSlimeTrail = True)    
    # robot.behavior = Brain(lib.sm_simple(-4*math.pi))
    # robot.behavior = RotateTSM(-2 * math.pi)
   
def brainStart():
    robot.behavior.start()

# def brainStop():
#     robot.behavior.start()

def step():
    sensor_input = io.SensorInput()

    action = robot.behavior.step(sensor_input)
    action.execute()
    io.done(robot.behavior.isDone())    
