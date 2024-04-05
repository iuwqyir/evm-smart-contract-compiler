const { compileContract } = require('./rust-compiler');

(async () => {
  const result = await compileContract();
  console.log(JSON.parse(result));
})()
