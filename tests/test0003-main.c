#include <stdio.h>

#include "test0003.h"
#include "test0003-snapshot.h"

int main() {
    long long bar = 321;
    long long baz = 67;
    snapshot_foo(stdout, &bar, &baz);
}
