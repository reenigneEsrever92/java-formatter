class Labels {
    void method() {
        outer:
        for (int i = 0; i < 10; i++) {
            if (i % 2 == 0) {
                inner:
                for (int j = 0; j < 10; j++) {
                    System.out.println(j);
                }
            }
        }
    }
}