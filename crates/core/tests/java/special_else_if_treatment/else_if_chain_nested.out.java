class ElseIfChain {
    void m(int a, int b, int c) {
        if (a) {
            one();
        } else {
            if (b) {
                two();
            } else {
                if (c) {
                    three();
                } else {
                    four();
                }
            }
        }
    }
}
