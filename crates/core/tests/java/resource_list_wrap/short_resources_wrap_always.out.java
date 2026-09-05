class Db {
    void load() {
        try (
            Connection conn = DriverManager.getConnection(url);
            Statement stmt = conn.createStatement()) {
            process(stmt);
        }
    }
}
