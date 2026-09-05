class Svc {
    void writeFile(String path, byte[] data) throws IOException,
            FileNotFoundException,
            SecurityException {
        Files.write(Paths.get(path), data);
    }
}
