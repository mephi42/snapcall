#include <stdio.h>
#include <stdlib.h>

#include "test.h"
#include "test0002.h"
#include "test0002-replay.h"

int main() {
    ASSERT_EQ(334, replay_foo_1());
    ASSERT_EQ(929, replay_foo_2());
}
