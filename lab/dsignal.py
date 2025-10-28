"""
Signals, represented implicitly, with plotting and combinations.
"""

from cffi import FFI
import os
import sys
import pickle
import math
import lib601.util as util

import lib601.gw as gw

ffi = FFI()

# Read the C header file
header_file = "/course/lab/sm.h"
with open(header_file) as f:
    ffi.cdef(f.read())

# Load the shared library
libname = "/course/lab/libsm.so"
lib = ffi.dlopen(libname)

# define size of graphing window 
# graphwidth = 400 # old value
graphwidth = 570
graphheight = 300

# NOTE: Ideally, graphwidth should be tuned to the number of samples in such a way
# that samples are an integer number of pixels apart on the screen.
# 570 seems to be just right for 250 samples (1000 steps and subsample 4).
# Samples are two pixels apart and there is space on the left for caption.
# Adjusting window width to get integer pixel spacing has now been
# automated in __init__ method of class GraphCanvas of gw.py

class Signal:
    """
    Represent infinite signals.  This is a generic superclass that
    provides some basic operations.  Every subclass must provide a
    C{sample} method.

    Be sure to start idle with the C{-n} flag, if you want to make
    plots of signals from inside idle.
    """

    def __init__(self, cobj):
        self._c = cobj
    
    __w = None
    """ Currently active plotting window.  Not for users."""

    def plot(self, start = 0, end = 100, newWindow = 'Signal value versus time',
             color = 'blue', parent = None, ps = None,
             xminlabel = 0, xmaxlabel = 0,
             yOrigin = None): # bkph
        """
        Make a plot of this signal.
        @param start: first value to plot; defaults to 0
        @param end: last value to plot; defaults to 100; must be > start
        @param newWindow: makes a new window with this value as title,
        unless the value is False, in which case it plots the signal
        in the currently active plotting window
        @param color: string specifying color of plot; all simple
        color names work
        @param parent: An instance of C{tk.tk}.  You probably should
        just leave this at the default unless you're making plots
        from another application.
        @param ps: If not C{None}, then it should be a pathname;
             we'll write a postscript version of this graph to that path.
        """
        samples = [self.sample(i) for i in range(start, end)]
        if len(samples) == 0:
            raise Exception, 'Plot range is empty'
        if yOrigin == None:
            minY = min(samples)
        else:
            minY = yOrigin
        maxY = max(samples)
        if maxY == minY:
            margin = 1.0
        else:
#           margin = (maxY - minY) * 0.05
            margin = 0 # override bkph
        
        if newWindow == True or newWindow == False:
            title = 'Signal value vs time'
        else:
            title = newWindow
            
        if parent:
            # Make a window under a different tk parent
            w = gw.GraphingWindow(\
                     graphwidth, graphheight, start, end,
                     minY-margin, maxY+margin, title, parent,
                     xminlabel = xminlabel, xmaxlabel = xmaxlabel) # bkph
        else:
            # Use this class's tk instance
            if  newWindow or Signal.__w == None:
                Signal.__w = gw.GraphingWindow(\
                     graphwidth, graphheight, start, end,
                     minY-margin, maxY+margin, title,
                     xminlabel = xminlabel, xmaxlabel = xmaxlabel) # bkph
            w = Signal.__w
            
        w.graphDiscrete(lambda n: samples[n - start], color)
        if ps:
            w.postscript(ps)

    def sample(self, n):        
        return lib.sig_sample(self._c, n)
