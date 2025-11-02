#include "data/example.hpp"

#include <stdio.h>

int main() {
    printf("Hello World\n");
    onyx::User::Buffer buf = {};
    onyx::User user = onyx::User::Deserialize(buf);
}
