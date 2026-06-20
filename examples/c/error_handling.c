/*
 * Error handling — how to check and respond to Glider C API errors.
 *
 * All API functions return SUCCESS (85) or FAILURE (0). This example
 * shows how to handle each failure scenario.
 *
 *   make error_handling && ./error_handling
 */

#include <stdio.h>
#include <stdlib.h>
#include "glider-api.h"

#define DISPLAY_WIDTH  1600
#define DISPLAY_HEIGHT 1200

int main(void) {
    /* --- Connection errors --- */

    printf("Attempting to connect...\n");
    Display *display = glider_open();
    if (!display) {
        fprintf(stderr, "Device not found.\n");
        fprintf(stderr, "Steps to fix:\n");
        fprintf(stderr, "  1. Confirm the Glider is plugged in via USB.\n");
        fprintf(stderr, "  2. On Linux: add a udev rule (see README) or run\n");
        fprintf(stderr, "       sudo chmod 0666 /dev/hidraw*\n");
        fprintf(stderr, "  3. Confirm VID/PID with: lsusb | grep 1209\n");
        return 1;
    }
    printf("Connected successfully.\n");

    /* --- Command errors --- */

    printf("Setting display mode...\n");
    Rect screen = {0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT};
    RESPONSE r = glider_set_mode(display, FAST_MONO_NO_DITHER, screen);

    if (r != SUCCESS) {
        fprintf(stderr, "set_mode failed (FAILURE). Retrying once...\n");
        r = glider_set_mode(display, FAST_MONO_NO_DITHER, screen);
        if (r != SUCCESS) {
            fprintf(stderr, "Retry failed. Check firmware version.\n");
            glider_close(display);
            return 1;
        }
        printf("Retry succeeded.\n");
    } else {
        printf("Mode set successfully.\n");
    }

    glider_close(display);
    return 0;
}
