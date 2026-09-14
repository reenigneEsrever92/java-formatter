class Svc {
    void writeFile(String path, byte[] data) throws IOException {
        Files.write(Paths.get(path), data);
    }

    Svc(String path, byte[] data, int size) throws IOException {
        this.size = size;
    }
}

