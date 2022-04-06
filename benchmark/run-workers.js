const { doRsTaskFromWorkers } = require('..');

console.time('from js workers');
doRsTaskFromWorkers();
process.on('exit', () => {
  console.timeEnd('from js workers');
});