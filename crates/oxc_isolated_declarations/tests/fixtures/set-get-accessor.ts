// Correct
class Cls {
  get a() {
    return 1;
  }
  set a() {
    return;
  }

  get b(): string {
  }
  set b(v) {
  }

  private get c() {}
  private set c() {}

  accessor d: string;
  private accessor e: string;
  private static accessor f: string;
}

// Incorrect
class ClsBad {
  get a() {
    return;
  }
  set a(v) {
  }
}
