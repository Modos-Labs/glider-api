/*
 * Redraw — force a hard refresh to clear ghosting.
 *
 *   make redraw && ./redraw
 */

#include <stdio.h>
#include "glider-api.h"

#define DISPLAY_WIDTH  1600
#define DISPLAY_HEIGHT 1200

int main(void) {
    Display *display = glider_open();
    if (!display) {
        fprintf(stderr, "Could not connect to display.\n");
        return 1;
    }

    printf("Redrawing top-left 200x200 region...\n");
    Rect region = {0, 0, 200, 200};
    if (glider_redraw(display, region) != SUCCESS) {
        fprintf(stderr, "redraw failed.\n");
        glider_close(display);
        return 1;
    }
    printf("OK\n");

    printf("Redrawing full screen...\n");
    Rect screen = {0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT};
    if (glider_redraw(display, screen) != SUCCESS) {
        fprintf(stderr, "redraw failed.\n");
        glider_close(display);
        return 1;
    }
    printf("OK\n");

    glider_close(display);
    return 0;
}
