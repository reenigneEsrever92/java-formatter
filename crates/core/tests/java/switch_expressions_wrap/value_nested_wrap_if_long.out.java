class A {
    int pick(int x, int y) {
        int z = switch (x) {
            case 1 -> switch (y) { case 2 -> 5; default -> 6; };
            default -> 0;
        };
        return z;
    }
}
