class CommentedDoWhile {
    void test(List<Entry> entries, Entry entry) {
        do {
            entries.add(entry);
        } while (// the list is still empty
                entries.isEmpty()
        );
    }
}

