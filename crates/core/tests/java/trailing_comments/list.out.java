class List {
    void run() {
        consume(
            a, // after a
            b /* after b */,
            c);
        consume(
            d,
            e // last
        );
    }
}

