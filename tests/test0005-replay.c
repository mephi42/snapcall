#include <stdio.h>

#include "test.h"
#include "test0005.h"
#include "test0005-replay.h"

int main() {
    ASSERT_EQ(388, replay_foo_1());
}
