from brain import Brain, lib

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
