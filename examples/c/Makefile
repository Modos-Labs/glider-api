CC      = gcc
CFLAGS  = -I../../include -Wall -Wextra
LIB_DIR = ../../target/debug
LIB     = $(LIB_DIR)/libglider_api.so
LDFLAGS = -Wl,-rpath,$(abspath $(LIB_DIR))

TARGETS = getting_started redraw mode_guide multi_zone error_handling

all: $(TARGETS)

%: %.c
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $< $(LIB)

clean:
	rm -f $(TARGETS)

.PHONY: all clean
