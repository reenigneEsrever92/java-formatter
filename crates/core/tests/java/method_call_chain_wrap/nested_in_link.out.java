class F {
    void m() {
        Config c = Config.builder()
                .name(LocalizedText.builder()
                    .text("Hello", Locale.GERMAN)
                    .text("World", Locale.ENGLISH)
                    .build())
                .build();
    }
}
