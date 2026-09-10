class F {
    void m() {
        C c = B.builder()
                .types(List.of(P.builder()
                    .key(A)
                    .active(true)
                    .build(), P.builder()
                    .key(B)
                    .active(false)
                    .build()))
                .build();
    }
}
