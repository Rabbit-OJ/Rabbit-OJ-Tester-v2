#include <iostream>
#include <cstring>
using namespace std;

int main() {
    for (int i = 0; i < 10; i++) {
        int* a = new int[10000000];
        memset(a, 0, sizeof(int) * 10000000);
    }

    while(1) {}
    return 0;
}