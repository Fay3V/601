import math
import lib601.sm as sm
from soar.io import io
import lib601.gfx as gfx
import lib601.util as util
import lib601.sonarDist as sonarDist

from brain import Brain, lib

reload(gfx)

######################################################################
#
#            Brain SM
#
######################################################################

desiredRight = 0.5
forwardVelocity = 0.1
k = -1000

# No additional delay
class Sensor(sm.SM):
    def getNextValues(self, state, inp):
        d_o = sonarDist.getDistanceRight(inp.sonars)
        print d_o
        return (state, d_o)

# inp is the distance to the right
class WallFollower(sm.SM):
    def getNextValues(self, state, inp):
        err = (desiredRight - inp)
        return (state, io.Action(forwardVelocity, k * err))

################
# Your code here
################

# mySM = sm.Cascade(Sensor(), WallFollower())
mySM = Brain(lib.sm(desiredRight , k))

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
    robot.behavior = mySM
    # robot.behavior.start(traceTasks = robot.gfx.tasks())
    robot.behavior.start()

def step():
    sensors = io.SensorInput()
    print sensors.odometry.theta
    action = robot.behavior.step(sensors)
    print action
    action.execute()

def brainStart():
    # Do this to be sure that the plots are cleared whenever you restart
    robot.gfx.clearPlotData()

def brainStop():
    pass

def shutdown():
    pass

