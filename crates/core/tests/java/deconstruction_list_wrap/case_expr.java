class Deconstruction {
    int f(Object o) {
        int r = switch (o) {
            case Point(int alpha, int beta) -> handle();
            default -> 0;
        };
        return r;
    }
}
