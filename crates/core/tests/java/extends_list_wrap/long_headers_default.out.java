class Alpha extends AbstractAlpha implements Named, Sized, Registry<Alpha>, Marked {
    int x;
}

interface Beta extends FirstInterface, SecondInterface, ThirdInterface {
    void run();
}
