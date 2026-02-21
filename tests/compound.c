int main(void) {
    int a = 10;
    a += 5;
    a -= 3;
    a *= 2;
    a /= 4;
    a %= 3;

    int b = 255;
    b &= 15;
    b |= 48;
    b ^= 7;
    b <<= 1;
    b >>= 2;

    return a + b;
}
