class Point {
    int x;
    int y;
    int sum() {
        return x + y;
    }
}

interface Marker {
    int CONSTANT = 42;

    void apply();
}

record Pair(int a, int b) {
    int total() {
        return a + b;
    }
}
