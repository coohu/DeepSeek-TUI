#!/usr/bin/env node

const { run } = require("../scripts/run");

run("deepseek").catch((error) => {
  console.error("Failed to start deepseek:", error.message);
  process.exit(1);
});
