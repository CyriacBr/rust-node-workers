console.time("load");
const start = Date.now();
const tsMorph = require("ts-morph");
while (Date.now() - start < 1000) {}
console.timeEnd("load");

function fib(n) {
  if (n <= 1) {
    return 1;
  }
  return fib(n - 1) + fib(n - 2);
}

function task(payload) {
  console.log("task start");
  const val = fib(payload.value);
  console.log("task end: ", val);
}

const readline = require("readline");
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

let payloadStr = "";
let payload = null;
let payloadStart = null;
rl.on("line", function (line) {
  switch (line) {
    case "PAYLOAD_END":
      payload = JSON.parse(payloadStr);
      payloadStr = "";
      console.log('payload received in', (Date.now() - payloadStart), 'ms');
      payloadStart = null;
      console.log('payload :>> ', payload);
      console.log("PAYLOAD_OK");
      break;
    case "WORK":
      task(payload);
      console.log("OK");
      break;
    default: {
      if (line.startsWith("PAYLOAD_CHUNK:")) {
        if (!payloadStart) {
          payloadStart = Date.now();
        }
        payloadStr += line.replace("PAYLOAD_CHUNK:", "").trim();
      }
      break;
    }
  }
});

console.log("READY");
