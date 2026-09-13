class CommentedTrailing {
    void test(List<Entry> entries, Entry entry) {
        if (entries.contains(entry) /* trailing */) {
            entries.add(entry);
        }
    }
}
