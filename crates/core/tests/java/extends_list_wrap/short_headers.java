class Holder implements Runnable, Serializable {
}

interface Marker extends Runnable, Serializable {
}

enum Level implements Named, Sized {
    LOW, HIGH;
}

record Point(int x, int y) implements Named, Sized {
}
