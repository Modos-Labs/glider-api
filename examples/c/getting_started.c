/*
 * Getting started with the Glider C API.
 *
 * Connects to a Modos e-ink display and sets the entire screen to fast
 * monochrome mode. Run this first to confirm your setup is working.
 *
 *   make getting_started && ./getting_started
 */

#include <stdio.h>
#include "glider-api.h"

#define DISPLAY_WIDTH  1600
#define DISPLAY_HEIGHT 1200

int main(void) {
    Display *display = glider_open();
    if (!display) {
        fprintf(stderr, "Could not connect to display.\n");
        fprintf(stderr, "Check that the Glider is plugged in and you have\n");
        fprintf(stderr, "permission to access the HID device (see README).\n");
        return 1;
    }

    Rect screen = {0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT};
    if (glider_set_mode(display, FAST_MONO_NO_DITHER, screen) != SUCCESS) {
        fprintf(stderr, "set_mode failed.\n");
        glider_close(display);
        return 1;
    }

    printf("Done — the display is now in fast monochrome mode.\n");
    glider_close(display);
    return 0;
}
