#include <stdio.h>

#include "test.h"
#include "test0001.h"
#include "test0001-replay.h"

int main() {
    ASSERT_EQ(388, replay_foo_1());
    ASSERT_EQ(6176, replay_foo_2());
}
