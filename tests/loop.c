int main(void) {
    int i = 0;

    // while loop
    while (i < 10) {
        i = i + 1;
        if (i == 5)
            continue;
        if (i == 8)
            break;
    }

    // do-while loop
    int j = 0;
    do {
        j = j + 1;
    } while (j < 5);

    // for loop
    int sum = 0;
    for (int k = 0; k < 10; k = k + 1) {
        if (k == 3)
            continue;
        if (k == 7)
            break;
        sum = sum + k;
    }

    return sum;
}
