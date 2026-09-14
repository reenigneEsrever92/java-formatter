class CommentInSimpleLambda {
    void test() {
        Runnable single = () -> {run();};
        Runnable commented = () -> {
            // comment before
            run();
        };
        Runnable multi = () -> {
            run();
            stop();
        };
    }
}

