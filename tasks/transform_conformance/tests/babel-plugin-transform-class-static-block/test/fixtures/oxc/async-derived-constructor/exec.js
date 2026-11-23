var Socket = class extends EventTarget {
  static {
    this.CONNECTING = 0;
  }

  constructor() {
    super();
    queueMicrotask(async () => {
      await Promise.resolve();
      this.ready = true;
    });
  }
};

const socket = new Socket();
expect(Socket.name).toBe("Socket");
expect(Socket.CONNECTING).toBe(0);
expect(socket).toBeInstanceOf(EventTarget);
expect(socket.ready).toBeUndefined();

return new Promise(resolve => setTimeout(resolve, 0)).then(() => {
  expect(socket.ready).toBe(true);
});
