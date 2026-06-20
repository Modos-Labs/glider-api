/*
 * Multi-zone layout — set different modes for different screen regions.
 *
 * Divides the 1600x1200 Glider display into three zones:
 *
 *   +-------------------+----------------+
 *   |                   |                |
 *   |  Left half        |  Top-right     |
 *   |  FAST_MONO_       |  AUTO_NO_      |
 *   |  NO_DITHER        |  DITHER        |
 *   |  (terminal)       |  (reading)     |
 *   |                   +----------------+
 *   |                   |  Bottom-right  |
 *   |                   |  FAST_MONO_    |
 *   |                   |  BAYER         |
 *   |                   |  (gaming)      |
 *   +-------------------+----------------+
 *
 *   make multi_zone && ./multi_zone
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

    int mid_x = DISPLAY_WIDTH  / 2;
    int mid_y = DISPLAY_HEIGHT / 2;

    /* Left half — terminal, code editor, drawing canvas */
    Rect left = {0, 0, mid_x, DISPLAY_HEIGHT};
    if (glider_set_mode(display, FAST_MONO_NO_DITHER, left) != SUCCESS) {
        fprintf(stderr, "set_mode failed for left zone.\n");
        glider_close(display);
        return 1;
    }
    printf("Left zone  (%dx%dpx): FAST_MONO_NO_DITHER\n",
           left.x1 - left.x0, left.y1 - left.y0);

    /* Top-right — browser, maps, reading app */
    Rect top_right = {mid_x, 0, DISPLAY_WIDTH, mid_y};
    if (glider_set_mode(display, AUTO_NO_DITHER, top_right) != SUCCESS) {
        fprintf(stderr, "set_mode failed for top-right zone.\n");
        glider_close(display);
        return 1;
    }
    printf("Top-right  (%dx%dpx): AUTO_NO_DITHER\n",
           top_right.x1 - top_right.x0, top_right.y1 - top_right.y0);

    /* Bottom-right — game or video content */
    Rect bottom_right = {mid_x, mid_y, DISPLAY_WIDTH, DISPLAY_HEIGHT};
    if (glider_set_mode(display, FAST_MONO_BAYER, bottom_right) != SUCCESS) {
        fprintf(stderr, "set_mode failed for bottom-right zone.\n");
        glider_close(display);
        return 1;
    }
    printf("Bottom-right (%dx%dpx): FAST_MONO_BAYER\n",
           bottom_right.x1 - bottom_right.x0, bottom_right.y1 - bottom_right.y0);

    printf("\nDone — three zones configured.\n");
    glider_close(display);
    return 0;
}
