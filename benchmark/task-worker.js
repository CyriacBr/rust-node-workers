console.time("load");
const start = Date.now();
const tsMorph = require("ts-morph");
while (Date.now() - start < 1000) {}
console.timeEnd("load");

const { bridge } = require('../dist/bridge');

function fib(n) {
  if (n <= 1) {
    return 1;
  }
  return fib(n - 1) + fib(n - 2);
}

bridge({
  fib: (payload) => {
    console.log("fib task start");
    const val = fib(payload.value);
    console.log("fib task end: ", val);
  },
  getUser: () => {
    return {
      name: 'Foo',
      age: 50,
      phones: ['a', 'b']
    };
  }
});
