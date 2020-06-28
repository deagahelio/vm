int main()
{
    int numbers[10] = {1, 1};
    for (int counter = 2; counter < 10; counter++)
        numbers[counter] = numbers[counter - 2] + numbers[counter - 1];
    //*((char*) 0x100000) = 255;
    for (int i = 0; i < 640*360*4; i++)
        *((int*) 0x100000 + i) = i%256;
}