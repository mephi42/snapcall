#include <stdio.h>

#include "test0001-snapshot.h"

int main() {
    snapshot_foo(stdout, 321, 67);
    snapshot_foo(stdout, 867, 5309);
}
