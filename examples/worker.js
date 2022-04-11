const { bridge } = require('../dist/bridge');

function fib(n) {
  if (n <= 1) {
    return 1;
  }
  return fib(n - 1) + fib(n - 2);
}

const { Project } = require('ts-morph');
function getInterfaces(filePath) {
  const project = new Project();
  const source = project.addSourceFileAtPath(filePath);
  const interfaces = source.getInterfaces();
  const results = [];
  for (const interface of interfaces) {
    const members = interface.getMembers();
    const props = [];
    for (const member of members) {
      const key = member.compilerNode.name.text;
      const type = member.compilerNode.type?.getText();
      props.push({ key, type });
    }
    results.push({
      name: interface.getName(),
      props,
    });
  }
  return results;
}

bridge({
  fib: (payload) => {
    console.log(`fib ${payload} task start`);
    const val = fib(payload);
    console.log(`fib ${payload} task end: `, val);
  },
  fib2: (payload) => {
    return fib(payload);
  },
  ping: () => {
    console.log(`pong at ${new Date()}`);
  },
  getUser: () => {
    return {
      name: 'Foo',
      age: 50,
      phones: ['a', 'b']
    };
  },
  error: () => {
    throw new Error('task failed');
  },
  getInterfaces
});

