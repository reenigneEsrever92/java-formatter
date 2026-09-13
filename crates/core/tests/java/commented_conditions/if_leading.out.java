class CommentedIf {
    void test(List<Entry> entries, Entry entry) {
        if (// the entry may already be in the list we are rebuilding
                // and we only want to add it once
                entries.stream()
                .noneMatch(other -> other.getId().equals(entry.getId()))
        ) {
            entries.add(entry);
        }
    }
}
