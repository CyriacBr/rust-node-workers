import readline from 'readline';

type Task = <T>(payload?: T) => object | null;
type Tasks = Record<string, Task>;
interface Options {
  debug?: boolean;
}
type Payload = {_inner_payload?: any};

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
  let payload: Payload | null = null;
  let payloadStart: number | null = null;
  rl.on("line", function (line) {
    switch (line) {
      case "PAYLOAD_END":
        payload = JSON.parse(payloadStr);
        if (payload?._inner_payload) {
          payload = payload?._inner_payload;
        }
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
          const res = task?.(payload);
          if (res) {
            const str = JSON.stringify(res);
            const chunks = str.match(/.{1,1000}/g) || [];
            for (const chunk of chunks) {
              console.log(`RESULT_CHUNK: ${chunk}`);
            }
          }
          console.log("OK");
        }
        break;
      }
    }
  });

  console.log("READY");
}
