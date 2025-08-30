from glider_api import *

d = Display()

# Set the lefthand side of the screen to 1-bit black and white. This is mode
# is currently only accessible via the API. In the demo video, this mode is used
# for drawing & using the terminal.
d.set_mode(Mode.FastMonoNoDither, Rect(0, 0, 800, 1200))

# Set the right-top corner of the screen to the "Reading" mode, which uses a 
# 1-bit mode when the content is updating and then updates regions of gray once
# the content stops changing. In the demo video, this mode is used for maps.
d.set_mode(Mode.AutoNoDither, Rect(800, 0, 1600, 600))

# Set the right-bottom corner of the screen to the "Browsing" mode, which is
# 1-bit but uses dither to approximate gray values. In the demo video, this mode
# is used for SuperTuxCart.
d.set_mode(Mode.FastMonoBayer, Rect(800, 600, 1600, 1200))