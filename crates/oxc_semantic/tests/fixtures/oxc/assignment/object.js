let read = {}, write = {};
({ A = read, B: write, C: { D: write, E = read } } = read);
