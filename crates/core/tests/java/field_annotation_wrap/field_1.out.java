class F {
    @Deprecated int old;

    @Column(name = "first", nullable = false)
    @Transient
    private static final String NAME = "x";

    int plain;

    @VeryLongAnnotationName(someArgument = "a long value here")
    private static final SomeVeryLongTypeNameWithLongLength A_REALLY_LONG_FIELD_NAME = new SomeVeryLongTypeNameWithLongLength();
}
