#include <stdio.h>

#include "test0004.h"
#include "test0004-snapshot.h"

int main() {
    float bar_val = 321;
    float *bar = &bar_val;
    float baz_val = 67;
    float *baz = &baz_val;
    snapshot_foo(stdout, &bar, &baz);
}
