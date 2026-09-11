class Patterns {
    void check(Object o) {
        if (o instanceof Point(int x, int y)) {
            System.out.println(x + y);
        }
        if (o instanceof Pair(String first, Integer second)) {
            System.out.println(first);
        }
    }
}