class F {
    void m() {
        use(a, LocalizedText.builder().text("Hello", Locale.GERMAN).text("World", Locale.ENGLISH).build());
    }
}
