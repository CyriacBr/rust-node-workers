"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.bridge = void 0;
const readline_1 = __importDefault(require("readline"));
function bridge(tasks, opts = {}) {
    const rl = readline_1.default.createInterface({
        input: process.stdin,
        output: process.stdout,
        terminal: false,
    });
    let debugOn = opts === null || opts === void 0 ? void 0 : opts.debug;
    function debug(...args) {
        if (debugOn)
            console.log.call(console, ...args);
    }
    let payloadStr = "";
    let payload = null;
    let payloadStart = null;
    rl.on("line", function (line) {
        switch (line) {
            case "PAYLOAD_END":
                payload = JSON.parse(payloadStr);
                payloadStr = "";
                debug("payload received in", Date.now() - payloadStart, "ms");
                payloadStart = null;
                debug("payload :>> ", payload);
                console.log("PAYLOAD_OK");
                break;
            default: {
                if (line.startsWith("PAYLOAD_CHUNK:")) {
                    if (!payloadStart) {
                        payloadStart = Date.now();
                    }
                    payloadStr += line.replace("PAYLOAD_CHUNK:", "").trim();
                }
                else if (line.startsWith("CMD:")) {
                    const cmd = line.replace("CMD:", "").trim();
                    const task = tasks[cmd];
                    debug("executing command: ", cmd);
                    task === null || task === void 0 ? void 0 : task(payload);
                    console.log("OK");
                }
                break;
            }
        }
    });
    console.log("READY");
}
exports.bridge = bridge;
//# sourceMappingURL=bridge.js.map