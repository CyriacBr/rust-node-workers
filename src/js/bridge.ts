import readline from 'readline';

type Task = <T>(payload?: T) => object | null;
type Tasks = Record<string, Task>;
interface Options {
  debug?: boolean;
}

export function bridge(tasks: Tasks, opts: Options = {}) {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
  });

  let debugOn = opts?.debug;
  function debug(...args: any[]) {
    if (debugOn) console.log.call(console, ...args);
  }

  let payloadStr = "";
  let payload: object | null = null;
  let payloadStart: number | null = null;
  rl.on("line", function (line) {
    switch (line) {
      case "PAYLOAD_END":
        payload = JSON.parse(payloadStr);
        payloadStr = "";
        debug("payload received in", Date.now() - payloadStart!, "ms");
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
        } else if (line.startsWith("CMD:")) {
          const cmd = line.replace("CMD:", "").trim();
          const task = tasks[cmd];
          debug("executing command: ", cmd);
          task?.(payload);
          console.log("OK");
        }
        break;
      }
    }
  });

  console.log("READY");
}
