class Patterns {
    void check(Object o) {
        if (o instanceof final String s) {
            System.out.println(s);
        }
    }
}