class Constructs {

    void method(
            String alpha,
            String beta,
            String gamma,
            String delta) {
        int[] data = new int[]{
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8};
        call(
                alpha(),
                beta(),
                gamma(),
                delta(),
                epsilon(),
                zeta());
        int chain = alpha()
                .beta()
                .gamma()
                .delta()
                .epsilon()
                .zeta();
    }
}