class Deconstruction {
    void f(Object o) {
        switch (o) {
            case String s -> handle();
            case Point(int alpha, int beta) when alpha > 0 -> handle();
            case 1, 2 -> other();
            default -> {}
        }
    }
}
