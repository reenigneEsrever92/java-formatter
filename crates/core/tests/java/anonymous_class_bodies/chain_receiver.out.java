class ChainReceiver {
    void use() {
        new Thread(new Runnable() {
            public void run() {
                go();
            }
        }).start();
    }
}

