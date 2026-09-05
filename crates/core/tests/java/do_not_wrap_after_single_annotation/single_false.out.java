class F {
    @Deprecated
    int single;

    @Deprecated
    @SuppressWarnings("unchecked")
    int pair;

    void m() {
        @Deprecated
        int local;
    }

    @Deprecated
    class Nested {}
}
