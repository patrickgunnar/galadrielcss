class Spinner {
    #spinnerFrames = ["-", "\\", "|", "/"];
    #currentFrame = 0;
    #interval = null;

    start(message) {
        this.#interval = setInterval(() => {
            process.stdout.write(
                `\r${this.#spinnerFrames[this.#currentFrame]} ${message}`
            );

            this.#currentFrame =
                (this.#currentFrame + 1) % this.#spinnerFrames.length;
        }, 100);
    }

    stop() {
        if (this.#interval) {
            clearInterval(this.#interval);
            process.stdout.write("\r");
            this.#interval = null;
        }
    }
}

module.exports = { Spinner };
