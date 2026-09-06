class Box<T extends Comparable<? super T> & Serializable> {
    <U extends Comparable<U> & Cloneable> U m() {
        return null;
    }
}
