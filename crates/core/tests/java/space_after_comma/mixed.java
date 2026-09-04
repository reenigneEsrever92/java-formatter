class A implements I1, I2 {
    @Ann(a = 1, b = 2)
    void m(int a, String b) throws E1, E2 {
        f(a, b, c);
        int[] arr = {1, 2, 3};
        int x = 1, y = 2;
        xs.forEach((u, v) -> use(u, v));
    }
}

record R(int a, int b) {
}
