console.time('load');
const start = Date.now();
const tsMorph = require('ts-morph');
while ((Date.now() - start) < 1000){}
console.timeEnd('load');

console.log('task start');
const val = fib(40);
console.log('task end: ', val);

function fib(n) {
  if (n <= 1) {
    return 1;
  }
  return fib(n - 1) + fib(n - 2);
}
