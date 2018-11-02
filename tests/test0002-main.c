#include <stdio.h>

#include "test0002.h"
#include "test0002-snapshot.h"

int main() {
    quux = 321;
    snapshot_foo(stdout, 6, 7);
    quux = 867;
    snapshot_foo(stdout, 53, 0x9);
}
