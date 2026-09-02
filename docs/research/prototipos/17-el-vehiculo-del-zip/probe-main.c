#include <stdio.h>
#include <unistd.h>
extern const unsigned char blob_start __asm("section$start$__DATA$__blob");
int main(int argc, char **argv) {
    fprintf(stderr, "flipchart-probe: alive, blob[0]=%u\n", (unsigned)blob_start);
    return 0;
}
