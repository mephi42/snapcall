#include "test0004.h"

float foo(float **bar, float **baz) {
    return **bar + **baz;
}
