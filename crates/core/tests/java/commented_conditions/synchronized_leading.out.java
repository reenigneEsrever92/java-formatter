class CommentedSynchronized {
    void test(Object lock, List<Entry> entries, Entry entry) {
        synchronized (// the lock guards the list
                lock
        ) {
            entries.add(entry);
        }
    }
}
