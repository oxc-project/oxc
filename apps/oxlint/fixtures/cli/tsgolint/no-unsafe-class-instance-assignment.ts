class Session {
  constructor(readonly id: string) {}
}

class GuestSession extends Session {}

class SessionRecord {
  constructor(readonly id: string) {}
}

const validSession: Session = new Session("guest");
const session: Session = { id: "guest" };
const sessions: Session[] = [new SessionRecord("guest")];
const guestSessions: Session[] = [new GuestSession("guest")];

export {};
