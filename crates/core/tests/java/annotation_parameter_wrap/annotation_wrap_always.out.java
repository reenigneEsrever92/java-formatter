class F {
    @CacheControl(maxAge = 3600,
        scopes = {"private"})
    void cached() {}

    @RequestMapping(path = "/some/long/path",
        method = RequestMethod.GET,
        produces = "application/json",
        consumes = {"application/json", "application/xml"})
    void handler() {}
}
