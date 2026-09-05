class F {
    void m(
        @Deprecated
        int a,
        int b) {}

    void n(
        @A
        @B
        int a,
        int b) {}
}
