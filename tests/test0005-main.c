#include <stdio.h>

#include "test0005.h"
#include "test0005-snapshot.h"

int main() {
    struct quux quux = { 321, 67 };
    snapshot_foo(stdout, &quux);
}
