class F {
    @Deprecated
    void old() {}

    @Override
    @SuppressWarnings("unchecked")
    public String toString() {
        return "x";
    }

    @Anno(arg = 1)
    @Another
    private static long id = 1;

    void plain() {}
}
