int main()
{
    int numbers[10];
    numbers[0] = 1;
    numbers[1] = 1;
    for (int counter = 2; counter < 10; counter++)
        numbers[counter] = numbers[counter - 2] + numbers[counter - 1];
    return numbers[9];
}