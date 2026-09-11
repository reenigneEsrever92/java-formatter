class Deconstruction {
    String f(Object o) {
        return switch (o) {
            case Point(int x,
                    // the y
                    int y) -> x + y + "";
            default -> "";
        };
    }

    record Point(int x, int y) {
    }
}
