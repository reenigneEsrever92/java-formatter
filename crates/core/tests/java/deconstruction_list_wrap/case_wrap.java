class Deconstruction {
    void f(Object o) {
        switch (o) {
            case Point(int alphaComponent, int betaComponent, int gammaComponent) -> handle();
            default -> {}
        }
    }
}
