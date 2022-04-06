const { doRsTaskFromJsCb } = require('..');
const task = require('./task');

console.time('from js callback');
doRsTaskFromJsCb(task);
process.on('exit', () => {
  console.timeEnd('from js callback');
});
