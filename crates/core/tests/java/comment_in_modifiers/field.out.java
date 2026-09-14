class Field {
    @Version
    @Column(name = "revision")
    // TODO change to long (incl. respective db columns)
    private int revision = 0;

    @OverridingMethodsMustInvokeSuper
    protected <T extends AggregateRoot<ID>> T copyNonModifiableFields(T target) {
        return target;
    }
}

