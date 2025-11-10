import math
import lib601.sm as sm
from soar.io import io
import lib601.gfx as gfx
import lib601.util as util
import lib601.sonarDist as sonarDist


from brain import Brain, lib


######################################################################
#
#            Brain SM
#
######################################################################

desiredRight = 0.4
forwardVelocity = 0.1

# No additional delay
class Sensor(sm.SM):
    def getNextValues(self, state, inp):
        v = sonarDist.getDistanceRight(inp.sonars)
        print 'Dist from robot center to wall on right', v
        return (state, v)

# inp is the distance to the right
class WallFollower(sm.SM):
    startState = None
    def getNextValues(self, state, inp):
        pass

################
# Your code here
################

# k1 = 10
# k2 = -9.93
# k1 = 30
# k2 = -29.74
# k1 = 100
# k2 = -97.33
k1 = 300
k2 = -271.6721311475485

# sensorMachine = Sensor()
# sensorMachine.name = 'sensor'
# mySM = sm.Cascade(sensorMachine, WallFollower())
mySM = Brain(lib.sm(desiredRight , k1, k2))

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
    # robot.gfx.addStaticPlotSMProbe(y=('rightDistance', 'sensor',
    #                                   'output', lambda x:x))
    robot.behavior = mySM
    # robot.behavior.start(traceTasks = robot.gfx.tasks())
    robot.behavior.start()

def step():
    robot.behavior.step(io.SensorInput()).execute()
    io.done(robot.behavior.isDone())

def brainStop():
    pass
