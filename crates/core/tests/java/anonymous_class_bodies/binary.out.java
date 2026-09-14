class Binary {
    boolean use(Object a) {
        return a == new Object() {
            @Override
            public String toString() {
                return "x";
            }
        };
    }
}

