
from dsignal import Signal, lib

######################################################################
##  Make a state machine model using primitives and combinators
######################################################################


# Plots the sequence of distances when the robot starts at distance
# initD from the wall, and desires to be at distance 0.7 m.  Time step
# is 0.1 s.  Parameter k is the gain;  end specifies how many steps to
# plot. 

initD = 1.5


def plot(k, end = 50):
	d = Signal(lib.sig(0.1, initD, k, 0.7))
	d.plot(0, end, newWindow = 'Gain ' +str(k))

# def plotD(k, end = 50):
#   d = ts.TransducedSignal(sig.ConstantSignal(0.7),
#                           wallFinderSystem(0.1, initD, k))
#   d.plot(0, end, newWindow = 'Gain '+str(k))

