class Logger {
    #getCurrentTime() {
        const now = new Date();
        const hours = String(now.getHours()).padStart(2, "0");
        const minutes = String(now.getMinutes()).padStart(2, "0");
        const seconds = String(now.getSeconds()).padStart(2, "0");
        const milliseconds = String(now.getMilliseconds()).padStart(3, "0");

        return `${hours}:${minutes}:${seconds}.${milliseconds}`;
    }

    makeBold(msg) {
        return `\x1b[1m${msg}\x1b[0m`;
    }

    now(message, bold = false) {
        const now = bold
            ? this.makeBold(this.#getCurrentTime())
            : this.#getCurrentTime();

        console.log(`[${now}]: ${message}`);
    }

    message(message, bold = false) {
        const msg = bold ? this.makeBold(message) : message;

        console.log(msg);
    }
}

module.exports = { Logger };
