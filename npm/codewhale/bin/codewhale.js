#!/usr/bin/env node

const { runCodeWhale } = require("../scripts/run");

runCodeWhale().catch((error) => {
  console.error("Failed to start deepseek:", error.message);
  process.exit(1);
});
