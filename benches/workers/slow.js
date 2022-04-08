// artificial slow startup
const start = Date.now();
while (Date.now() - start < 1000) {}

const { bridge } = require("../../dist/bridge");

function fib(n) {
  if (n <= 1) {
    return 1;
  }
  return fib(n - 1) + fib(n - 2);
}

bridge({
  fib: (payload) => {
    console.log(`fib ${payload} task start`);
    const val = fib(payload);
    console.log(`fib ${payload} task end: `, val);
  },
});
