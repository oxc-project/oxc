declare interface User { id: number; }
declare function getUser(): Promise<User>;
async function fetchUser(): Promise<User> {
  return getUser();
}

export {};
