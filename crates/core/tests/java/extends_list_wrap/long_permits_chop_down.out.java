sealed class Alpha extends AbstractAlpha implements Named,
        Sized,
        Registry<Alpha> permits FirstPermitted,
        SecondPermitted,
        ThirdLongPermitted {
    int x;
}

sealed interface Beta extends FirstInterface,
        SecondInterface,
        ThirdInterface permits FirstImpl,
        SecondImpl,
        ThirdLongImpl {
    void run();
}

