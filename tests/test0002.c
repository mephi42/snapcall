#include "test0002.h"

long quux;

long foo(long bar, long baz) {
    long tmp;

    tmp = bar + baz;
    return tmp + quux;
}
