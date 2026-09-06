class Box <T> {}

interface I <U> {}

record R <V>(V x) {}

class WithMethod <T> {
    <U> U m(U u) {
        return u;
    }
}
