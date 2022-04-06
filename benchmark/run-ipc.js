const { doRsTaskFromJsCb } = require('..');
const task = require('./task');
const ipc = require('node-ipc').default;

ipc.config.id = 'hello';
ipc.config.retry = 1000;

ipc.connectTo('world', function () {
  ipc.of.world.on('connect', function () {
    ipc.log('## connected to world ##', ipc.config.delay);
    ipc.of.world.emit('app.message', {
      id: ipc.config.id,
      message: 'hello',
    });
  });
  ipc.of.world.on('disconnect', function () {
    ipc.log('disconnected from world');
  });
  ipc.of.world.on('app.message', function (data) {
    task();
  });

  console.log(ipc.of.world.destroy);
});

console.time('from js callback');
doRsTaskFromJsCb(task);
process.on('exit', () => {
  console.timeEnd('from js callback');
});
