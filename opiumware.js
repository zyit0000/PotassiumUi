const Net = require("net");
const Zlib = require("zlib");

const PORTS = ["8392", "8393", "8394", "8395", "8396", "8397"];

function connectOnce(port, timeoutMs = 800) {
  return new Promise((resolve, reject) => {
    const socket = Net.createConnection({ host: "127.0.0.1", port: Number(port) }, () => {
      socket.setTimeout(0);
      resolve(socket);
    });
    socket.setTimeout(timeoutMs, () => {
      socket.destroy(new Error("timeout"));
    });
    socket.on("error", reject);
  });
}

function deflateUtf8(str) {
  return new Promise((resolve, reject) => {
    Zlib.deflate(Buffer.from(str, "utf8"), (err, compressed) => {
      if (err) reject(err);
      else resolve(compressed);
    });
  });
}

async function execute(code, port) {
  const portsToUse = port === "ALL" ? PORTS : [String(port)];

  // "NULL" is used by Potassium as a connection probe (attach-to-port test).
  const isProbe = code === "NULL";

  if (port === "ALL" && !isProbe) {
    const successPorts = [];
    let lastError = null;

    for (const p of portsToUse) {
      try {
        const socket = await connectOnce(p);
        try {
          const compressed = await deflateUtf8(code);
          await new Promise((resolve, reject) =>
            socket.write(compressed, (err) => (err ? reject(err) : resolve()))
          );
          successPorts.push(p);
        } finally {
          socket.end();
        }
      } catch (err) {
        lastError = err;
      }
    }

    if (successPorts.length === 0) {
      return `Failed to connect on all ports${lastError ? `: ${lastError.message}` : ""}`;
    }
    if (successPorts.length === 1) {
      return `Successfully connected to Opiumware on port: ${successPorts[0]}`;
    }
    return `Successfully executed on ports: ${successPorts.join(", ")}`;
  }

  // Single port (or probe): mirror the original behavior (stop after first success).
  for (const p of portsToUse) {
    try {
      const socket = await connectOnce(p);
      try {
        if (!isProbe) {
          const compressed = await deflateUtf8(code);
          await new Promise((resolve, reject) =>
            socket.write(compressed, (err) => (err ? reject(err) : resolve()))
          );
        }
      } finally {
        socket.end();
      }
      return `Successfully connected to Opiumware on port: ${p}`;
    } catch (_) {}
  }

  return "Failed to connect on all ports";
}

async function checkPort(port) {
  try {
    const socket = await connectOnce(port, 400);
    socket.end();
    return true;
  } catch (_) {
    return false;
  }
}

async function attachAny() {
  // Scan for first reachable port.
  return execute("NULL", "ALL");
}

async function attachToPort(port) {
  return execute("NULL", String(port));
}

module.exports = {
  PORTS,
  execute,
  checkPort,
  attachAny,
  attachToPort,
};

if (require.main === module) {
  // Example:
  execute("OpiumwareScript print('hello')", "ALL").then(console.log).catch(console.error);
}
