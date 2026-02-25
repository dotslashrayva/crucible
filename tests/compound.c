int main(void) {
    int x = 1;
    int y = 2;
    {
        int x = 10;
        y = x + y;
        {
            int x = 100;
            y = y + x;
        }
        y = y + x;
    }
    return y + x;
}
