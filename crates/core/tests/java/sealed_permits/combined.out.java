public sealed class Foo<T> extends Base implements I1, I2 permits A, B {
    T x;
}
