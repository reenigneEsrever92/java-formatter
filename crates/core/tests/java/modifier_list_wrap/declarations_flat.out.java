class A {
    @Deprecated
    public static final int MAX_VALUE = 100;

    public synchronized native int someMethod(int a, int b);

    @Override
    public String toString() {
        return "x";
    }

    @Anno
    class Nested {}
}
