class Statement {
    void run() {
        int a = 1; // local trailing
        int b = 2; /* block trailing */
        if (a > b) { // if brace trailing
            a++; // inc trailing
        } else { // else brace trailing
            b--; // dec trailing
        }
    }
}

