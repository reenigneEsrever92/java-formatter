class Deconstruction {
    void f(Object o) {
        switch (o) {
            case Point( int alpha, int beta ) -> handle();
            default -> {}
        }
    }
}
