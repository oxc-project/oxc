import type { Nested, Repository, Result } from "./types";

interface User {
  id: number;
  name: string;
}

// Clean-only fixture: defaults applied, constraints satisfied,
// members match after instantiation. tsc reports nothing here.
const ok_success: Result<number> = { ok: true, value: 10 };
const ok_failure: Result<number> = { ok: false, error: "not found" };
const ok_custom_error: Result<string, number> = { ok: false, error: 404 };

const ok_repo: Repository<number, User> = {
  find(id: number): User | null {
    return id === 1 ? { id: 1, name: "first" } : null;
  },
  save(entity: User): number {
    return entity.id;
  },
};

const ok_nested: Nested<boolean> = { inner: { ok: true, value: true } };

export function describe_result(res: Result<number>): string {
  return res.ok ? "ok" : res.error;
}

export const ok_saved_id: number = ok_repo.save({ id: 2, name: "second" });
