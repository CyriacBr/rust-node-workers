const { doRsTaskFromCmd } = require('..');

console.time('from js cmd');
doRsTaskFromCmd();
process.on('exit', () => {
  console.timeEnd('from js cmd');
});