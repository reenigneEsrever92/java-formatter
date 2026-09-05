class F {
    void m(
        @Deprecated
        int a,
        String b) {}

    void n(
        @A
        @B
        int a,
        @C
        String b,
        int c) {}

    void p(int plain) {}

    void q(
        @VeryLongAnnotationName(someArgument = "a long value here")
        SomeVeryLongTypeName parameter,
        int other) {}
}
