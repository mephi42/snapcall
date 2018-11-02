#ifndef TEST_H
#define TEST_H

#include <stdlib.h>
#include <stdio.h>

#define ASSERT_EQ(x, y) do { \
    typeof(x) _x = (x); \
    typeof(y) _y = (y); \
    if (_x != _y) { \
        fprintf(stderr, "%s:%d: assertion failed\n", __FILE__, __LINE__); \
        abort(); \
    } \
} while (0)

#endif
