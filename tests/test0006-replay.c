#include <stdio.h>

#include "test.h"
#include "test0006.h"
#include "test0006-replay.h"

int main() {
    ASSERT_EQ(388, replay_foo_1());
}
