// artificial slow startup
const start = Date.now();
while (Date.now() - start < 1000) {}

function fib(n) {
  if (n <= 1) {
    return 1;
  }
  return fib(n - 1) + fib(n - 2);
}
fib(parseInt(process.argv[2], 10));
