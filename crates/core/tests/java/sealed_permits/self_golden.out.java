public sealed interface PermitA permits One, Two, Three {
    void run();
}

public sealed abstract class PermitB<X> extends Base implements Named, Sized permits Four, Five, Six {
    abstract X value();
}
