class A {
    int pick(int x) {
        int z = switch (x) { case 1 -> firstBranch(); case 2 -> secondBranch(); default -> fallbackBranch(); };
        return z;
    }
}
