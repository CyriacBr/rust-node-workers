console.time('load');
const tsMorph = require('ts-morph');
console.timeEnd('load');

module.exports = function () {
  console.log('task start');
  const val = fib(40);
  console.log('task end: ', val);
};

function fib(n) {
  if (n <= 1) {
    return 1;
  }
  return fib(n - 1) + fib(n - 2);
}