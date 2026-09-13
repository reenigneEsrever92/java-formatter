class CommentedWhile {
    void test(List<Entry> entries, Entry entry) {
        while (
            // keep looping while the entry is still missing
            entries.stream()
                .noneMatch(other -> other.getId().equals(entry.getId()))
        ) {
            entries.add(entry);
        }
    }
}
