#include <stdio.h>
#include <stdlib.h>

#include "test.h"
#include "test0003.h"
#include "test0003-replay.h"

int main() {
    ASSERT_EQ(388, replay_foo_1());
}
