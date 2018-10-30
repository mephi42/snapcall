#include <stdio.h>
#include <stdlib.h>

#include "test0001.h"
#include "test0001-replay.h"

int main() {
    if (replay_foo_1() != 388)
        return EXIT_FAILURE;
    if (replay_foo_2() != 6176)
        return EXIT_FAILURE;
}
