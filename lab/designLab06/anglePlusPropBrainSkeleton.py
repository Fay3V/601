import math
import lib601.sm as sm
from soar.io import io
import lib601.gfx as gfx
import lib601.util as util
import lib601.sonarDist as sonarDist


from brain import lib, ffi


######################################################################
#
#            Brain SM
#
######################################################################

desiredRight = 0.4
forwardVelocity = 0.1

# No additional delay.
# Output is a sequence of (distance, angle) pairs
class Sensor(sm.SM):
   def getNextValues(self, state, inp):
       v = sonarDist.getDistanceRightAndAngle(inp.sonars)
       print 'Dist from robot center to wall on right', v[0]
       if not v[1]:
           print '******  Angle reading not valid  ******'
       return (state, v)


# inp is a tuple (distanceRight, angle)
class WallFollower(sm.SM):
    startState = None
    def getNextValues(self, state, inp):
        pass

class Brain(object):
    def __init__(self, cobj):
        self._c = cobj

    def start(self):
        lib.sm_reset(self._c)

    def isDone(self):
        return lib.sm_is_done(self._c)

    def step(self, sensor_input):
        v = sonarDist.getDistanceRightAndAngle(sensor_input.sonars)
        print 'getDistanceRightAndAngle ', v
        if not v[1]:
           print '******  Angle reading not valid  ******'

        # typedef struct Tuple2_bool_double {
        #     bool _0;

        #     double _1;
        # } Tuple2_bool_double_t;

        # typedef struct AnglePropInput {
        #     double distance;

        #     Tuple2_bool_double_t angle;
        # } AnglePropInput_t;

        # print inp
        input = ffi.new("AnglePropInput_t *")
        input.distance = v[0]
        if not v[1]:
            input.angle._0 = False
            input.angle._1 = 0.0 
        else:
            input.angle._0 = True
            input.angle._1 = v[1] 

        output = lib.sm_step(self._c, input[0])
        action = io.Action(output.fvel, output.rvel)
        return action

    def run(self, n):
        val = lib.sm_run(self._c, n)
        return [val.ptr[i] for i in range(val.len)]


################
# Your code here
################
k3 = 3
k4 = 1.09

# sensorMachine = Sensor()
# sensorMachine.name = 'sensor'
# mySM = sm.Cascade(sensorMachine, WallFollower())
mySM = Brain(lib.sm(desiredRight , k3, k4))

######################################################################
#
#            Running the robot
#
######################################################################


def plotDist():
    func = lambda: sonarDist.getDistanceRight(io.SensorInput().sonars)
    robot.gfx.addStaticPlotFunction(y=('d_o', func))

def setup():
    robot.gfx = gfx.RobotGraphics(drawSlimeTrail=False)
    plotDist()
    robot.gfx.addStaticPlotSMProbe(y=('rightDistance', 'sensor',
                                      'output', lambda x:x[0]))
    robot.behavior = mySM
    robot.behavior.start()

def step():
    robot.behavior.step(io.SensorInput()).execute()

def brainStop():
    pass
