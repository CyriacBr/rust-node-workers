const { bridge } = require('../dist/bridge');

bridge({
  ping: () => {
    console.log(`pong at ${new Date()}`);
  },
});

