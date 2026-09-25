/*
 * Standby — put the display into low-power standby and wake it again.
 *
 * The display blanks the panel and stops updating while in standby.
 * Call glider_exit_standby() to resume normal operation.
 *
 *   make standby && ./standby
 */

#include <stdio.h>
#include <unistd.h>
#include "glider-api.h"

#define STANDBY_SECS 3

int main(void) {
    Display *display = glider_open();
    if (!display) {
        fprintf(stderr, "Could not connect to display.\n");
        return 1;
    }

    printf("Entering standby...\n");
    if (glider_enter_standby(display) != SUCCESS) {
        fprintf(stderr, "enter_standby failed.\n");
        glider_close(display);
        return 1;
    }
    printf("Display is in standby. Waiting %d seconds...\n", STANDBY_SECS);

    sleep(STANDBY_SECS);

    printf("Exiting standby...\n");
    if (glider_exit_standby(display) != SUCCESS) {
        fprintf(stderr, "exit_standby failed.\n");
        glider_close(display);
        return 1;
    }
    printf("Done — display resumed.\n");

    glider_close(display);
    return 0;
}
