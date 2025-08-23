export class WorkspaceResolver {
    async invite() {
      try {
        await using lock = await acquire(lockFlag);
      } catch {
      }
    }
  }