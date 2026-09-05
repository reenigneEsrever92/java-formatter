class F {
    void m() {
        @Deprecated
        int old = 1;

        @A(value = 2)
        @B
        final String name = "x";

        int plain = 3;

        @VeryLongAnnotationName(someArgument = "a long value here")
        SomeVeryLongTypeNameWithLongLength variableName = new SomeVeryLongTypeNameWithLongLength();
    }
}