#!/usr/bin/env node

const { exec } = require("child_process");

const controller = typeof AbortController !== "undefined" ? new AbortController() : { abort: () => {} };
const { signal } = controller;

exec("static-file-http-server", { signal }, (error, stdout, stderr) => {
  stdout && console.log(stdout);
  stderr && console.error(stderr);
  if (error !== null) {
    console.log(`exec error: ${error}`);
  }
});

process.on("SIGTERM", () => {
  controller && controller.abort();
});

process.on("SIGINT", () => {
  controller && controller.abort();
});
    