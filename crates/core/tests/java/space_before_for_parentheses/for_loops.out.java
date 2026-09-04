import java.util.List;

class A {
    void m(List<String> xs) {
        for(int i = 0; i < 10; i++) {
            f(i);
        }
        for(String s : xs) {
            g(s);
        }
    }

    void f(int i) {}

    void g(String s) {}
}
