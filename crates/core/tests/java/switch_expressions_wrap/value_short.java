class A {
    int pick(int x) {
        int z = switch (x) {
            case 1 -> 5;
            case 2 -> 6;
            default -> 0;
        };
        return z;
    }
}
