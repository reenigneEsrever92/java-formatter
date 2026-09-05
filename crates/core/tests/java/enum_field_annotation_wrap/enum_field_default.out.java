enum E {
    @Deprecated OLD(1),
    @A("x") @B("y") MANY(2),
    PLAIN(3),
    @VeryLongAnnotationName(someArgument = "a long value here") LONG_CONSTANT_NAME_WITH_MANY_CHARACTERS(100, 200, 300, 400, 500, 600, 700);
}
