class Ternary {
    Object use(boolean b) {
        return b ? new Runnable() {
            public void run() {
                go();
            }
        } : null;
    }
}

