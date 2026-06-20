/*
 * Mode guide — cycle through every display mode on the full screen.
 *
 * Run with a Glider connected to see how each Mode affects the display.
 * Pauses between modes so you can observe the difference.
 *
 *   make mode_guide && ./mode_guide
 */

#include <stdio.h>
#include <unistd.h>
#include "glider-api.h"

#define DISPLAY_WIDTH  1600
#define DISPLAY_HEIGHT 1200
#define PAUSE_SECS     3

typedef struct {
    MODE        mode;
    const char *name;
    const char *description;
} ModeEntry;

static const ModeEntry MODES[] = {
    { FAST_MONO_NO_DITHER,   "FastMonoNoDither",   "Fastest refresh, hard black/white. Best for terminals and code." },
    { FAST_MONO_BAYER,       "FastMonoBayer",      "Fast refresh with Bayer dithering. Best for games and moving content." },
    { FAST_MONO_BLUE_NOISE,  "FastMonoBlueNoise",  "Fast refresh, blue-noise dither. Best for images with gradients." },
    { FAST_GREY,             "FastGrey",           "4-level greyscale. Slowest refresh, best image quality. Best for reading." },
    { AUTO_NO_DITHER,        "AutoNoDither",       "Hybrid: 1-bit while updating, greyscale when idle. Best for maps/reading." },
    { AUTO_ERROR_DIFFUSION,  "AutoErrorDiffusion", "Like AutoNoDither with error-diffusion dithering for smoother transitions." },
};

int main(void) {
    Display *display = glider_open();
    if (!display) {
        fprintf(stderr, "Could not connect to display.\n");
        return 1;
    }

    Rect screen = {0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT};
    int n = sizeof(MODES) / sizeof(MODES[0]);

    for (int i = 0; i < n; i++) {
        printf("\n%s\n  %s\n", MODES[i].name, MODES[i].description);
        if (glider_set_mode(display, MODES[i].mode, screen) != SUCCESS) {
            fprintf(stderr, "set_mode failed for %s.\n", MODES[i].name);
            glider_close(display);
            return 1;
        }
        sleep(PAUSE_SECS);
    }

    printf("\nDone — all modes demonstrated.\n");
    glider_close(display);
    return 0;
}
