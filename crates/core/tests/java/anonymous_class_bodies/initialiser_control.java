class InitialiserControl {
    Runnable field = new Runnable() {
        public void run() {
            go();
        }
    };

    void use() {
        Runnable local = new Runnable() {
            public void run() {
                go();
            }
        };
        new Runnable() {
            public void run() {
                go();
            }
        };
        Runnable[] array = new Runnable[] {
            new Runnable() {
                public void run() {
                    go();
                }
            }
        };
    }

    Runnable make() {
        return new Runnable() {
            public void run() {
                go();
            }
        };
    }
}
