use node_workers::WorkerPool;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct Property {
  pub key: String,
  #[serde(alias = "type")]
  pub propType: String,
}
#[derive(Deserialize, Debug)]
struct Interface {
  pub name: String,
  pub props: Vec<Property>,
}

fn main() {
  // Create a pool of 4 node workers
  let mut pool = WorkerPool::setup(4);
  pool.with_debug(true);

  // Payloads
  let files = vec![
    Path::new("examples/user-files/user.ts")
      .canonicalize()
      .unwrap(),
    Path::new("examples/user-files/pet.ts")
      .canonicalize()
      .unwrap(),
  ];

  // execute the command "getInterfaces" on every file
  // each executed worker will return an array of interfaces (Vec<Interface>)
  let interfaces = pool
    .perform::<Vec<Interface>, _>("examples/worker", "getInterfaces", files)
    .unwrap();
  let interfaces: Vec<Interface> = interfaces
    .into_iter()
    .map(|x| x.unwrap())
    .flatten()
    .collect();
  println!("interfaces: {:#?}", interfaces);
}
