class Outer {
    sealed interface Inner permits Impl1, Impl2 {
        void run();
    }

    static final class Impl1 implements Inner {}

    static final class Impl2 implements Inner {}
}
