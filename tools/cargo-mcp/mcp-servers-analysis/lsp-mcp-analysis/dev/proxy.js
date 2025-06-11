// Pulled from https://stackoverflow.com/questions/35353164/how-can-i-log-and-proxy-all-stdio-to-a-sub-process
// Run: node shim.js

const { spawn } = require("child_process");
const { createWriteStream } = require("fs");

// Start another process and pipe its output to the console and to a log file
// Pass our standard input too
function startProcess(commandAndArguments, log) {
  var child = spawn(commandAndArguments[0], commandAndArguments.slice(1));
  child.stdout.pipe(process.stdout);
  child.stderr.pipe(process.stderr);
  process.stdin.pipe(child.stdin);
  const logStream = createWriteStream(log);
  logStream.write("Starting process: " + commandAndArguments.join(" ") + "\n");
  child.stdout.on("data", (data) => {
    logStream.write("Output: " + data + "\n");
  });
  child.stderr.on("data", (data) => {
    logStream.write("Error: " + data + "\n");
  });
  process.stdin.on("data", (data) => {
    logStream.write("Input: " + data + "\n");
  });
}

function agentShim() {
  //get command line args
  const args = process.argv.slice(2);
  startProcess(
    ["/Users/jonrad/projects/lsp-mcp/.venv/bin/jedi-language-server", ...args],
    "/tmp/shim.log",
  );
}

agentShim();
