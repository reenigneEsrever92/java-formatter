class CommentIf {
    void test() {
        list.forEach(item -> {
            // keep only items we do not have yet
            if (!persisted.contains(item.getId())) {
                add(item);
            }
        });
    }
}
